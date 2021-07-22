use std::io::Result;
use either::Either;

use super::bzstring::BZString;
use super::hash::Hash;
use super::version::Version;


pub trait Bsa {
    fn version(&self) -> Result<Version>;

    fn file_tree(&mut self) -> Result<Vec<BsaFile>>;
}

pub type FileId = Either<Hash, BZString>;
#[derive(Debug)]
pub enum BsaFile {
    File {
        name: FileId,
        compressed: bool,
        offset: u64,
    },
    Dir {
        name: FileId,
        files: Vec<BsaFile>,
    },
}
