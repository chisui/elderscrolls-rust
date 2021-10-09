use std::{fmt, fs::File, io::{BufReader, Result, Write}, path::Path, slice::Iter};
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BsaDir {
    pub id: EntryId,
    pub files: Vec<BsaFile>,
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
    pub id: EntryId,
    pub compressed: bool,
    pub offset: u64,
    pub size: usize,
}


pub fn open<B, P>(path: P) -> Result<B>
where
    B: BsaReader<In = BufReader<File>>,
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    B::read_bsa(buf)
}
pub trait BsaReader: Sized {
    type Header;
    type Root = Vec<BsaDir>;
    type In;

    fn read_bsa(input: Self::In) -> Result<Self>;

    fn header(&self) -> Self::Header;

    fn list(&mut self) -> Result<Self::Root>;

    fn extract<W: Write>(&mut self, file: &BsaFile, writer: W) -> Result<()>;
}
