use std::{
    io::{Read, Result},
    fmt,
    hash,
};
use bytemuck::{Zeroable, Pod};

use super::bin::{self, concat_bytes};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
pub struct Hash {
    left: u32,
    right: u32,
}
impl From<Hash> for u64 {
    fn from(h: Hash) -> u64 {
        (h.left as u64 >> 16) + h.right as u64
    }
}
impl From<u64> for Hash {
    fn from(n: u64) -> Self {
        Self {
            left: (n << 16) as u32,
            right: n as u32,
        }
    }
}
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}{:08x}", self.left, self.right)
    }
}
impl bin::Readable for Hash {
    fn read_here<R: Read>(reader: R, _: &()) -> Result<Self> {
        bin::read_struct(reader)
    }
}
impl hash::Hash for Hash {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.left);
        state.write_u32(self.right);
    }
}

pub fn hash_v10x(path: &str) -> Hash {
    let lower = path.to_lowercase();
    let right_sep = lower.replace('/', "\\");
    let (root, ext) = hash_v10x_parts(right_sep.as_str());

    Hash {
        left: concat_bytes([
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

        right: when(root.len() > 2, || hash_sdbm(&root[1 .. root.len() - 2]))
            .wrapping_add(hash_sdbm(ext)),
    }
}

fn hash_v10x_parts(chars: &str) -> (&[u8], &[u8]) {
    let bytes = chars.as_bytes();
    for (i, c) in chars.char_indices().rev() {
        match c {
            '\\' => break,
            '.'  => return (&bytes[0 .. i], &bytes[i .. bytes.len()]),
            _    => (),
        }
    }
    (bytes, &[])
}


pub fn hash_v100(bytes: &[u8]) -> Hash {
    let mid_point = bytes.len() >> 1;
    
    Hash {
        left: {
            let mut left = 0;
            for i in mid_point .. bytes.len() {
                let temp = (bytes[i] as u32) << (((i - mid_point) & 3) << 3);
                left = rot_right(left ^ temp, temp & 0x1F);
            }
            left
        },
        right: concat_bytes({
            let mut right: [u8; 4] = [0; 4];
            for i in 0 .. mid_point {
                right[i & 3] ^= bytes[i];
            }
            right
        }),
    }
}

/// http://www.partow.net/programming/hashfunctions/index.html#SDBMHashFunction
fn hash_sdbm(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0;
    for c in bytes {
        hash = hash.wrapping_mul(0x01003f) + *c as u32;
    }
    hash
}

const fn rot_right(value: u32, num_bits: u32) -> u32 {
    value << (32 - num_bits) | value >> num_bits
}

fn when<T: Default, F>(cond: bool, v: F) -> T 
where F: FnOnce() -> T {
    if cond { v() } else { T::default() }
}
