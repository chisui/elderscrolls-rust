#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization)]
#[macro_use]
pub mod bin;
pub mod read;
pub mod write;
pub mod str;
pub mod hash;
pub mod magicnumber;
pub mod version;
pub mod v001;
pub mod v10x;
pub mod v103;
pub mod v104;
pub mod v105;

use std::{
    io::{BufReader, Read, Seek, Write, Result},
    fs::File,
    path::Path,
};
use thiserror::Error;

use crate::{
    read::{BsaReader, BsaDir, BsaFile},
    bin::Readable,
};

pub use crate::{
    hash::Hash,
    version::{Version, Version10X},
    v001::V001,
    v103::V103,
    v104::V104,
    v105::V105,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum ForSomeBsaVersion<A001, A103, A104, A105> {
    #[error("{0}")] V001(A001),
    #[error("{0}")] V103(A103),
    #[error("{0}")] V104(A104),
    #[error("{0}")] V105(A105),
}
impl<A001, A103, A104, A105> ForSomeBsaVersion<A001, A103, A104, A105> {

    pub fn version(&self) -> Version {
        match self {
            ForSomeBsaVersion::V001(_) => Version::V001,
            ForSomeBsaVersion::V103(_) => Version::V10X(Version10X::V103),
            ForSomeBsaVersion::V104(_) => Version::V10X(Version10X::V104),
            ForSomeBsaVersion::V105(_) => Version::V10X(Version10X::V105),
        }
    }
}

pub type SomeBsaHeader = ForSomeBsaVersion<v001::Header, v103::Header, v104::Header, v105::Header>;
pub type SomeBsaReader<R> = ForSomeBsaVersion<v001::BsaReader<R>, v103::BsaReader<R>, v104::BsaReader<R>, v105::BsaReader<R>>;

pub fn open<P>(path: P) -> Result<SomeBsaReader<BufReader<File>>>
where P: AsRef<Path> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    read(buf)
}
pub fn read<R>(mut reader: R) -> Result<SomeBsaReader<R>>
where R: Read + Seek {
    let v = Version::read0(&mut reader)?;
    v.read(reader)
}

pub enum SomeBsaRoot {
    Dirs(Vec<BsaDir>),
    Files(Vec<BsaFile>),
}
impl<A001, A103, A104, A105> BsaReader for ForSomeBsaVersion<A001, A103, A104, A105> 
where
    A001: BsaReader<Header = v001::Header, Root = Vec<BsaFile>>,
    A103: BsaReader<Header = v103::Header, Root = Vec<BsaDir>>,
    A104: BsaReader<Header = v104::Header, Root = Vec<BsaDir>>,
    A105: BsaReader<Header = v105::Header, Root = Vec<BsaDir>>,
{
    type Header = SomeBsaHeader;
    type Root = SomeBsaRoot;

    fn header(&self) -> Self::Header {
        match self {
            ForSomeBsaVersion::V001(bsa) => SomeBsaHeader::V001(bsa.header()),
            ForSomeBsaVersion::V103(bsa) => SomeBsaHeader::V103(bsa.header()),
            ForSomeBsaVersion::V104(bsa) => SomeBsaHeader::V104(bsa.header()),
            ForSomeBsaVersion::V105(bsa) => SomeBsaHeader::V105(bsa.header()),
        }
    }

    fn list(&mut self) -> Result<SomeBsaRoot> {
        match self {
            ForSomeBsaVersion::V001(bsa) => bsa.list().map(SomeBsaRoot::Files),
            ForSomeBsaVersion::V103(bsa) => bsa.list().map(SomeBsaRoot::Dirs),
            ForSomeBsaVersion::V104(bsa) => bsa.list().map(SomeBsaRoot::Dirs),
            ForSomeBsaVersion::V105(bsa) => bsa.list().map(SomeBsaRoot::Dirs),
        }
    }

    fn extract<W: Write>(&mut self, file: &BsaFile, writer: W) -> Result<()> {
        match self {
            ForSomeBsaVersion::V001(bsa) => bsa.extract(file, writer),
            ForSomeBsaVersion::V103(bsa) => bsa.extract(file, writer),
            ForSomeBsaVersion::V104(bsa) => bsa.extract(file, writer),
            ForSomeBsaVersion::V105(bsa) => bsa.extract(file, writer),
        }
    }
}
