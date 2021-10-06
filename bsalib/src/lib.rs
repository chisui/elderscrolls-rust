#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization)]
#[macro_use]
mod bin;
mod str;
pub mod read;
pub mod write;
pub mod hash;
pub mod version;
pub mod v001;
pub mod v10x;
pub mod v103;
pub mod v104;
pub mod v105;

use std::io::{Read, Seek, Write, Result};
use bin::ReadableFixed;
use thiserror::Error;

use crate::read::{BsaReader, BsaDir, BsaFile};
pub use crate::{
    hash::Hash,
    version::{Version, Version10X, BA2Type},
    read::open,
    v001::{V001, BsaReaderV001, HeaderV001},
    v103::{V103, BsaReaderV103, HeaderV103},
    v104::{V104, BsaReaderV104, HeaderV104},
    v105::{V105, BsaReaderV105, HeaderV105},
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

pub type SomeBsaHeader = ForSomeBsaVersion<HeaderV001, HeaderV103, HeaderV104, HeaderV105>;
pub type SomeBsaReader<R> = ForSomeBsaVersion<BsaReaderV001<R>, BsaReaderV103<R>, BsaReaderV104<R>, BsaReaderV105<R>>;

pub enum SomeBsaRoot {
    Dirs(Vec<BsaDir>),
    Files(Vec<BsaFile>),
}
impl<R> BsaReader for ForSomeBsaVersion<BsaReaderV001<R>, BsaReaderV103<R>, BsaReaderV104<R>, BsaReaderV105<R>>
where R: Read + Seek {
    type Header = SomeBsaHeader;
    type Root = SomeBsaRoot;
    type In = R;

    fn read_bsa(mut reader: R) -> Result<Self> {
        let v = Version::read_fixed(&mut reader)?;
        v.read(reader)
    }


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
