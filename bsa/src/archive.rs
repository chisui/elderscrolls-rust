use std::io::{Result, Write, Seek};
use std::fmt;

use super::hash::Hash;
use super::version::Version;
use super::bin::DataSource;


#[derive(Debug, PartialEq, Eq)]
pub enum FileId {
    Hash(Hash),
    String(String),
}
impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileId::Hash(h)                   => write!(f, "#{}", h),
            FileId::String(s) => {
                write!(f, "{}", s.replace('\\', "/"))
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

pub struct BsaDirSource<D> {
    pub name: String,
    pub files: Vec<BsaFileSource<D>>,
}
impl<D> BsaDirSource<D> {
    pub fn new(name: String, files: Vec<BsaFileSource<D>>) -> Self {
        Self { name, files }
    }
}
pub struct BsaFileSource<D> {
    pub name: String,
    pub compressed: Option<bool>,
    pub data: D,
}
impl<D> BsaFileSource<D> {
    pub fn new(name: String, data: D) -> Self {
        Self { name, compressed: None, data}
    }
}
pub trait BsaWriter {
    type Options;
    fn write_bsa<DS, D, W>(opts: Self::Options, dirs: DS, out: W) -> Result<()>
    where
        D: DataSource,
        DS: IntoIterator<Item = BsaDirSource<D>>,
        W: Write + Seek;
}
