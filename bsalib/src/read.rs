use std::{
    io::{Result, Write},
    fmt,
    slice::Iter,
};
use crate::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryId {
    pub hash: Hash,
    pub name: Option<String>,
}
impl fmt::Display for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}", name.replace('\\', "/"))
        } else {
            write!(f, "#{}", self.hash)
        }
    }
}

pub trait BsaEntry {
    fn id(&self) -> EntryId;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BsaDir {
    pub hash: Hash,
    pub name: Option<String>,
    pub files: Vec<BsaFile>,
}
impl BsaEntry for BsaDir {
    fn id(&self) -> EntryId {
        EntryId {
            hash: self.hash,
            name: self.name.clone(),
        }
    }
}
impl<'a> IntoIterator for &'a BsaDir {
    type Item = &'a BsaFile;
    type IntoIter = Iter<'a, BsaFile>;
    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BsaFile {
    pub hash: Hash,
    pub name: Option<String>,
    pub compressed: bool,
    pub offset: u64,
    pub size: usize,
}
impl BsaEntry for BsaFile {
    fn id(&self) -> EntryId {
        EntryId {
            hash: self.hash,
            name: self.name.clone(),
        }
    }
}

pub trait BsaReader: Sized {
    type Header;

    fn header(&self) -> Self::Header;

    fn dirs(&mut self) -> Result<Vec<BsaDir>>;

    fn extract<W: Write>(&mut self, file: &BsaFile, writer: W) -> Result<()>;
}