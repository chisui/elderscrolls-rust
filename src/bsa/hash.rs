
pub struct Hash(u64);


#[doc = "Returns tes4's two hash values for filename."]
#[doc = "Based on TimeSlips code with cleanup and pythonization."]
pub fn hash_file(file_name: String) -> Option<Hash> {
    let name_sane = file_name.to_lowercase().replace('/', "\\");
    let (base, ext) = name_sane.rsplit_once('.')?;

    let hash1 = hash_part1(base) | hash_ext(ext);
    let hash2 = hash_part2(&base.as_bytes()[1 .. base.len() - 2]);
    let hash3 = hash_part2(ext.as_bytes());

    Some (Hash(hash1 + ((hash2 + hash3) << 32)))
}

fn hash_part1(base: &str) -> u64 {
    let chars = base.as_bytes();
    let mut hash1: u64 = chars[chars.len() - 1] as u8 as u64;
    hash1 |= (chars[chars.len() - 2] as u64) << 8;
    hash1 |= (base.len() as u8 as u64) << 16;
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
    let mut hash: u64 = 0;
    for c in chars {
        hash = (hash * 0x1003f) + (*c as u64);
    }
    hash
}
