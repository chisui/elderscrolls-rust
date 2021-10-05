use std::{fmt, hash};
use bytemuck::{Zeroable, Pod};

use crate::bin::concat_bytes;
use crate::bin::{derive_readable_via_pod, derive_writable_via_pod};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct Hash {
    low: u32,
    high: u32,
}
derive_readable_via_pod!(Hash);
derive_writable_via_pod!(Hash);
impl Hash {
    pub fn v001<S>(s: S) -> Self
    where S: AsRef<str> {
        let path = sanitize(s);
        let bytes = path.as_bytes();
        let mid_point = bytes.len() >> 1;
    
        Self {
            low: (mid_point .. bytes.len())
                .fold(0, |low, i| {
                    let temp = (bytes[i] as u32) << (((i - mid_point) & 3) << 3);
                    rot_high(low ^ temp, temp & 0x1F)
                }),
            high: concat_bytes({
                let mut high: [u8; 4] = [0; 4];
                for i in 0 .. mid_point {
                    high[i & 3] ^= bytes[i];
                }
                high
            }),
        }
    }


    pub fn v10x<S>(s: S) -> Self
    where S: AsRef<str> {
        let path = sanitize(s);
        let (root, ext) = hash_v10x_parts(path.as_bytes());

        Self {
            low: concat_bytes([
                root[root.len() - 1],
                when(root.len() > 2, || root[root.len() - 2]),
                root.len() as u8,
                root[0],
            ]) | match &*ext {
                b".nif" => 0x00008000,
                b".kf"  => 0x00000080,
                b".dds" => 0x00008080,
                b".wav" => 0x80000000,
                _       => 0x00000000,
            },

            high: when(root.len() > 2, || hash_sdbm(&root[1 .. root.len() - 2]))
                .wrapping_add(hash_sdbm(ext)),
        }
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}{:08x}", self.low, self.high)
    }
}
impl hash::Hash for Hash {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.low);
        state.write_u32(self.high);
    }
}

fn sanitize<S>(path: S) -> String
where S: AsRef<str> {
    path.as_ref()
        .to_lowercase()
        .replace('/', "\\")
}

fn hash_v10x_parts(bytes: &[u8]) -> (&[u8], &[u8]) {
    for (i, c) in (0 .. bytes.len()).zip(bytes).rev() {
        match *c as char {
            '\\' => break,
            '.'  => return (&bytes[0 .. i], &bytes[i .. bytes.len()]),
            _    => (),
        }
    }
    (bytes, &[])
}

/// http://www.partow.net/programming/hashfunctions/index.html#SDBMHashFunction
fn hash_sdbm(bytes: &[u8]) -> u32 {
    bytes.into_iter()
        .fold(0, |hash, c| hash.wrapping_mul(0x01003f) + *c as u32)
}

fn rot_high(value: u32, num_bits: u32) -> u32 {
    value.wrapping_shl(32 - num_bits) | value. wrapping_shl(num_bits)
}

fn when<T: Default, F>(cond: bool, v: F) -> T 
where F: FnOnce() -> T {
    if cond { v() } else { T::default() }
}
