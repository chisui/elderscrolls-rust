use std::fmt;

use super::bzstring::BZString;
use super::hash::Hash;


pub enum FileId {
    HashId(Hash),
    StringId(BZString),
}
impl fmt::Debug for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileId({})", self)
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileId::HashId(Hash(h))             => write!(f, "#{:016x}", h),
            FileId::StringId(BZString{ value }) => {
                let no_null: String = value.chars().take(value.len() - 2).collect();
                write!(f, "{}", no_null.replace('\\', "/"))
            },
        }
    }
}

#[derive(Debug)]
pub struct BsaDir {
    pub name: FileId,
    pub files: Vec<BsaFile>,
}

#[derive(Debug)]
pub struct BsaFile {
    pub name: FileId,
    pub compressed: bool,
    pub offset: u64,
}
