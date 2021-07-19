use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Hash(u64);

impl From<&str> for Hash {
    #[doc = "Returns tes4's two hash values for filename."]
    #[doc = "Based on TimeSlips code with cleanup and pythonization."]
    fn from(file_name: &str) -> Self {
        let (base0, ext) = hash_parts(file_name);
        let base = base0.as_bytes();

        let hash1 = hash_part1(base)
            | hash_ext(ext);
        let hash2 = hash_part2(&base[1 .. base.len() - 2])
            + hash_part2(ext.as_bytes());

        Hash(hash1 + (hash2 << 32))
    }
}

fn hash_parts(file_name: &str) -> (&str, &str) {
    file_name.rsplit_once('.').unwrap_or((file_name, ""))
}

fn hash_part1(chars : &[u8]) -> u64 {
    let mut hash1 = 0;
    hash1 |= chars[chars.len() - 1] as u8 as u64;
    hash1 |= (chars[chars.len() - 2] as u64) << 8;
    hash1 |= (chars.len() as u8 as u64) << 16;
    hash1 |= chars[0] as u64;
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
    let mut hash = 0;
    for c in chars {
        hash = (hash * 0x1003f) + (*c as u64);
    }
    hash
}
