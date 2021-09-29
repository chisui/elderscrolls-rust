use std::io::{Result, Read, Write, Seek};
use std::fmt;

use super::hash::Hash;
use super::version::Version;
use super::bzstring::BZString;


#[derive(Debug)]
pub enum FileId {
    HashId(Hash),
    StringId(BZString),
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileId::HashId(h)                   => write!(f, "#{}", h),
            FileId::StringId(BZString{ value }) => {
                write!(f, "{}", value.replace('\\', "/"))
            },
        }
    }
}

pub trait HasName {
    fn name<'a>(&'a self) -> &'a FileId;
}

#[derive(Debug)]
pub struct BsaDir {
    pub name: FileId,
    pub files: Vec<BsaFile>,
}
impl HasName for BsaDir {
    fn name<'a>(&'a self) -> &'a FileId {
        &self.name
    }
}

#[derive(Debug)]
pub struct BsaFile {
    pub name: FileId,
    pub compressed: bool,
    pub offset: u64,
    pub size: u32,
}
impl HasName for BsaFile {
    fn name<'a>(&'a self) -> &'a FileId {
        &self.name
    }
}

pub trait Bsa: fmt::Display + Sized {
    type Header;

    fn version(&self) -> Version;

    fn header(&self) -> Self::Header;

    fn read_dirs(&mut self) -> Result<Vec<BsaDir>>;

    fn extract<W: Write>(&mut self, file: BsaFile, writer: W) -> Result<()>;
}

pub struct BsaDirSource {
    pub name: String,
    pub files: Vec<BsaFileSource>,
}
pub struct BsaFileSource {
    pub name: String,
    pub compressed: Option<bool>,
    pub data: Box<dyn Read>,
}
pub trait BsaWriter {
    type Options;
    fn write<D, W>(opts: Self::Options, dirs: D, out: W) -> Result<()>
    where
        D: IntoIterator<Item = BsaDirSource> + Copy,
        W: Write + Seek;
}
