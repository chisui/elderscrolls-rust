use std::fmt;
use std::hash;
use bytemuck::{Pod, Zeroable};
use super::bzstring::BZString;

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Zeroable, Pod)]
pub struct Hash(pub u64);
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

impl From<BZString> for Hash {
    fn from(file_name: BZString) -> Self {
        Hash::from(&file_name)
    }
}
impl From<&BZString> for Hash {
    fn from(file_name: &BZString) -> Self {
        let lower = file_name.value.to_lowercase();
        let right_sep = lower.replace('/', "\\");
        Hash(hash_full(&right_sep))
    }
}


fn hash_full(path: &String) -> u64 {
    let (root, ext) = split_ext(path);
    hash_parts(root, ext)
}

pub fn hash_dir(dir: &String) -> u64 {
    hash_parts(dir.as_str(), "")
}

fn hash_parts(root: &str, ext: &str) -> u64 {
    let chars = root.as_bytes();
    let hash2 = when(chars.len() > 2,
        || hash_part2(&chars[1 .. chars.len() - 2]));
    ((hash2.wrapping_add(hash_part2(ext.as_bytes())) as u64) << 32) + hash_part1(chars, ext)
}

fn hash_part1(chars: &[u8], ext: &str) -> u64 {
    let mut hash1 = 0;
    hash1 |= chars[chars.len() - 1] as u64;
    hash1 |= when(chars.len() > 2,
        || (chars[chars.len() - 2] as u64) << 8);
    hash1 |= (chars.len() as u64) << 16;
    hash1 |= (chars[0] as u64) << 24;
    hash1 |= match &*ext {
        ".nif" => 0x00008000,
        ".kf"  => 0x00000080,
        ".dds" => 0x00008080,
        ".wav" => 0x80000000,
        _      => 0x00000000,
    };
    hash1
}

fn when<T: Default, F>(cond: bool, v: F) -> T 
where F: FnOnce() -> T {
    if cond { v() } else { T::default() }
}

fn hash_part2(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0;
    for c in bytes {
        hash = hash.wrapping_mul(0x1003f) + *c as u32;
    }
    hash
}

fn split_ext(path: &str) -> (&str, &str) {
    for (i, c) in path.char_indices().rev() {
        match c {
            '\\' => break,
            '.'  => return (&path[0 .. i], &path[i .. path.len()]),
            _    => ()
        }
    }
    (path, "")
}
