use std::{
    io::{Read, Write, Result},
    mem::size_of,
    fmt,
    hash,
};
use bytemuck::{Zeroable, Pod};

use super::bin::{self, concat_bytes, write_struct};


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct Hash {
    low: u32,
    high: u32,
}
impl Hash {
    pub fn v001<S>(s: S) -> Self
    where S: AsRef<str> {
        hash_v001(sanitize(s).as_bytes())
    }


    pub fn v10x<S>(s: S) -> Self
    where S: AsRef<str> {
        hash_v10x(sanitize(s).as_bytes())
    }
}

impl From<Hash> for u64 {
    fn from(h: Hash) -> u64 {
        (h.low as u64 >> 16) + h.high as u64
    }
}
impl From<u64> for Hash {
    fn from(n: u64) -> Self {
        Self {
            low: (n << 16) as u32,
            high: n as u32,
        }
    }
}
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}{:08x}", self.low, self.high)
    }
}
impl bin::Readable for Hash {
    fn read_here<R: Read>(reader: R, _: &()) -> Result<Self> {
        bin::read_struct(reader)
    }
}
impl bin::Writable for Hash {
    fn size(&self) -> usize { size_of::<Hash>() }
    fn write_here<W: Write>(&self, out: W) -> Result<()> {
        write_struct(self, out)
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

fn hash_v10x(bytes: &[u8]) -> Hash {
    let (root, ext) = hash_v10x_parts(bytes);

    Hash {
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


fn hash_v001(bytes: &[u8]) -> Hash {
    let mid_point = bytes.len() >> 1;
    
    Hash {
        low: {
            let mut low = 0;
            for i in mid_point .. bytes.len() {
                let temp = (bytes[i] as u32) << (((i - mid_point) & 3) << 3);
                low = rot_high(low ^ temp, temp & 0x1F);
            }
            low
        },
        high: concat_bytes({
            let mut high: [u8; 4] = [0; 4];
            for i in 0 .. mid_point {
                high[i & 3] ^= bytes[i];
            }
            high
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

fn rot_high(value: u32, num_bits: u32) -> u32 {
    value.wrapping_shl(32 - num_bits) | value. wrapping_shl(num_bits)
}

fn when<T: Default, F>(cond: bool, v: F) -> T 
where F: FnOnce() -> T {
    if cond { v() } else { T::default() }
}
