use std::fmt;
use std::hash;
use bytemuck::{Pod, Zeroable};
use super::bzstring::BZString;

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Zeroable, Pod)]
pub struct Hash(u64);
impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({:016x})", self.0)
    }
}
impl hash::Hash for Hash {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0)
    }
} 


impl From<&BZString> for Hash {
    #[doc = "Returns tes4's two hash values for filename."]
    #[doc = "Based on TimeSlips code with cleanup and pythonization."]
    fn from(file_name: &BZString) -> Self {
        let (base0, ext) = hash_parts(file_name.value.as_str());
        let base = base0.as_bytes();

        let hash1 = hash_part1(base)
            | hash_ext(ext);
        let mut hash2 = if base.len() >= 3 {
            hash_part2(&base[1 .. base.len() - 2])
        } else {
            0
        };
        hash2 = hash2.wrapping_add(hash_part2(ext.as_bytes()));

        Hash(hash1 + (hash2 << 32))
    }
}

fn hash_parts(file_name: &str) -> (&str, &str) {
    file_name.rsplit_once('.').unwrap_or((file_name, ""))
}

fn hash_part1(chars : &[u8]) -> u64 {
    let mut hash1 = 0;
    if chars.len() >= 1 {
        hash1 |= chars[chars.len() - 1] as u8 as u64;
    }
    if chars.len() >= 2 {
        hash1 |= (chars[chars.len() - 2] as u64) << 8;
    }
    hash1 |= (chars.len() as u8 as u64) << 16;
    if chars.len() >= 1 {
        hash1 |= chars[0] as u64;
    }
    hash1
}

fn hash_ext(ext: &str) -> u64 {
    match ext {
        ".kf"  => 0x80,
        ".nif" => 0x8000,
        ".dds" => 0x8080,
        ".wav" => 0x80000000,
        _ => 0,
    }
}

fn hash_part2(chars: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    for c in chars {
        hash = hash.wrapping_mul(0x1003f).wrapping_add(*c as u64);
    }
    hash
}
