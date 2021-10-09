use std::slice::{Iter, SliceIndex};
use std::path::Path;
use std::ops::Index;
use std::io::{BufReader, Result, Write};
use std::fs;
use std::fmt;

use crate::Hash;


/// Identifier for [`Dir`] and [`File`].
/// All directories and files of a bsa archive have at least a [`Hash`] to identify them.
/// Depending on the archive they may also have a name.
/// All official bsa archives contain names for each directory and files.
/// Games may refuse to load archives without names.
/// The file format itself permits directories and files without names though.
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

/// A diretory inside of a bsa archive.
/// If a name is present in the [`id`] it will be the complete path of the directory.
/// Subdirectories are represented by differend entries.
/// No official bsa archives contain empty directories, this isn't enforced by the file format though.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dir {
    pub id: EntryId,
    pub files: Vec<File>,
}
impl<'a> IntoIterator for &'a Dir {
    type Item = &'a File;
    type IntoIter = Iter<'a, File>;
    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}
impl<I: SliceIndex<[File]>> Index<I> for Dir {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.files.index(index)
    }
}

/// A file inside of a bsa archive.
/// If the file is [`compressed`] then [`size`] referes to the compressed data size.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct File {
    pub id: EntryId,
    pub compressed: bool,
    pub offset: u64,
    pub size: usize,
}

/// Open a bsa archive.
pub fn open<B, P>(path: P) -> Result<B>
where
    B: Reader<In = BufReader<fs::File>>,
    P: AsRef<Path>,
{
    let file = fs::File::open(path)?;
    let buf = BufReader::new(file);
    B::read_bsa(buf)
}
pub trait Reader: Sized {
    type Header;
    type Root = Vec<Dir>;
    type In;

    fn read_bsa(input: Self::In) -> Result<Self>;

    fn header(&self) -> Self::Header;

    fn list(&mut self) -> Result<Self::Root>;

    fn extract<W: Write>(&mut self, file: &File, writer: W) -> Result<()>;
}
