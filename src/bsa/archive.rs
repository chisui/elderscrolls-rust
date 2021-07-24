use std::fmt;

use super::hash::Hash;
use super::bzstring::BZString;


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
            FileId::HashId(h)             => write!(f, "#{}", h),
            FileId::StringId(BZString{ value }) => {
                write!(f, "{}", value.replace('\\', "/"))
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
    pub size: u32,
}
