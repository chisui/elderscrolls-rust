#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization)]
#[macro_use]
mod bin;
mod compress;
mod str;
mod read;
mod write;
mod hash;
mod version;
mod v001;
mod v10x;
mod v103;
mod v104;
mod v105;

use std::io::{self, Read, Seek, Write};
use bin::ReadableFixed;
use thiserror::Error;

pub use crate::hash::Hash;
pub use crate::version::{Version, Version10X, BA2Type};
pub use crate::read::{open, BsaReader, BsaDir, BsaFile, BsaEntry, EntryId};
pub use crate::write::{BsaDirSource, BsaFileSource, BsaWriter, list_dir};
pub use crate::v001::{V001, BsaReaderV001, HeaderV001, BsaWriterV001};
pub use crate::v10x::ToArchiveBitFlags;
pub use crate::v103::{V103, BsaReaderV103, HeaderV103, BsaWriterV103, ArchiveFlagV103};
pub use crate::v104::{V104, BsaReaderV104, HeaderV104, BsaWriterV104, ArchiveFlagV104};
pub use crate::v105::{V105, BsaReaderV105, HeaderV105, BsaWriterV105, ArchiveFlagV105};


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum ForSomeBsaVersion<A001, A10X> {
    #[error("{0}")] V001(A001),
    #[error("{0}")] V10X(A10X),
}
impl<A001, A103, A104, A105> ForSomeBsaVersion<A001, ForSomeBsaVersion10X<A103, A104, A105>> {
    pub fn version(&self) -> Version {
        match self {
            ForSomeBsaVersion::V001(_) => Version::V001,
            ForSomeBsaVersion::V10X(v) => Version::V10X(v.version()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum ForSomeBsaVersion10X<A103, A104, A105> {
    #[error("{0}")] V103(A103),
    #[error("{0}")] V104(A104),
    #[error("{0}")] V105(A105),
}
impl<A103, A104, A105> ForSomeBsaVersion10X<A103, A104, A105> {
    pub fn version(&self) -> Version10X {
        match self {
            ForSomeBsaVersion10X::V103(_) => Version10X::V103,
            ForSomeBsaVersion10X::V104(_) => Version10X::V104,
            ForSomeBsaVersion10X::V105(_) => Version10X::V105,
        }
    }
}

pub type SomeBsaHeaderV10X = ForSomeBsaVersion10X<HeaderV103, HeaderV104, HeaderV105>;
pub type SomeBsaHeader = ForSomeBsaVersion<HeaderV001, SomeBsaHeaderV10X>;

pub type SomeBsaReaderV10X<R> = ForSomeBsaVersion10X<BsaReaderV103<R>, BsaReaderV104<R>, BsaReaderV105<R>>;
pub type SomeBsaReader<R> = ForSomeBsaVersion<BsaReaderV001<R>, SomeBsaReaderV10X<R>>;

pub type SomeBsaWriterV10X = ForSomeBsaVersion10X<BsaWriterV103, BsaWriterV104, BsaWriterV105>;
pub type SomeBsaWriter = ForSomeBsaVersion<BsaWriterV001, SomeBsaWriterV10X>;

pub type SomeBsaRoot = ForSomeBsaVersion<Vec<BsaFile>, Vec<BsaDir>>;

impl<R> BsaReader for SomeBsaReader<R>
where R: Read + Seek {
    type Header = SomeBsaHeader;
    type Root = SomeBsaRoot;
    type In = R;

    fn read_bsa(mut reader: R) -> io::Result<Self> {
        Version::read_fixed(&mut reader)?
            .read_bsa(reader)
    }

    fn header(&self) -> Self::Header {
        match self {
            ForSomeBsaVersion::V001(bsa) => ForSomeBsaVersion::V001(bsa.header()),
            ForSomeBsaVersion::V10X(bsa) => ForSomeBsaVersion::V10X(bsa.header()),
        }
    }

    fn list(&mut self) -> io::Result<SomeBsaRoot> {
        match self {
            ForSomeBsaVersion::V001(bsa) => bsa.list().map(SomeBsaRoot::V001),
            ForSomeBsaVersion::V10X(bsa) => bsa.list().map(SomeBsaRoot::V10X),
        }
    }

    fn extract<W: Write>(&mut self, file: &BsaFile, writer: W) -> io::Result<()> {
        match self {
            ForSomeBsaVersion::V001(bsa) => bsa.extract(file, writer),
            ForSomeBsaVersion::V10X(bsa) => bsa.extract(file, writer),
        }
    }
}


impl<R> BsaReader for SomeBsaReaderV10X<R>
where R: Read + Seek {
    type Header = SomeBsaHeaderV10X;
    type Root = Vec<BsaDir>;
    type In = R;

    fn read_bsa(mut reader: R) -> io::Result<Self> {
        Version10X::read_fixed(&mut reader)?
            .read_bsa(reader)
    }

    fn header(&self) -> Self::Header {
        match self {
            ForSomeBsaVersion10X::V103(bsa) => ForSomeBsaVersion10X::V103(bsa.header()),
            ForSomeBsaVersion10X::V104(bsa) => ForSomeBsaVersion10X::V104(bsa.header()),
            ForSomeBsaVersion10X::V105(bsa) => ForSomeBsaVersion10X::V105(bsa.header()),
        }
    }

    fn list(&mut self) -> io::Result<Vec<BsaDir>> {
        match self {
            ForSomeBsaVersion10X::V103(bsa) => bsa.list(),
            ForSomeBsaVersion10X::V104(bsa) => bsa.list(),
            ForSomeBsaVersion10X::V105(bsa) => bsa.list(),
        }
    }

    fn extract<W: Write>(&mut self, file: &BsaFile, writer: W) -> io::Result<()> {
        match self {
            ForSomeBsaVersion10X::V103(bsa) => bsa.extract(file, writer),
            ForSomeBsaVersion10X::V104(bsa) => bsa.extract(file, writer),
            ForSomeBsaVersion10X::V105(bsa) => bsa.extract(file, writer),
        }
    }
}

impl BsaWriter for SomeBsaWriter {
    type Err = ForSomeBsaVersion<
        <BsaWriterV001 as BsaWriter>::Err,
        ForSomeBsaVersion10X<
            <BsaWriterV103 as BsaWriter>::Err,
            <BsaWriterV104 as BsaWriter>::Err,
            <BsaWriterV105 as BsaWriter>::Err,
        >,
    >;

    fn write_bsa<DS, D, W>(&self, dirs: DS, out: W) -> Result<(), Self::Err>
    where
        D: bin::DataSource,
        DS: IntoIterator<Item = BsaDirSource<D>>,
        W: Write + Seek {
            match self {
                ForSomeBsaVersion::V001(writer) => writer.write_bsa(dirs, out).map_err(ForSomeBsaVersion::V001),
                ForSomeBsaVersion::V10X(writer) => writer.write_bsa(dirs, out).map_err(ForSomeBsaVersion::V10X),
            }
    }
}


impl BsaWriter for SomeBsaWriterV10X {
    type Err = ForSomeBsaVersion10X<
        <BsaWriterV103 as BsaWriter>::Err,
        <BsaWriterV104 as BsaWriter>::Err,
        <BsaWriterV105 as BsaWriter>::Err,
    >;

    fn write_bsa<DS, D, W>(&self, dirs: DS, out: W) -> Result<(), Self::Err>
    where
        D: bin::DataSource,
        DS: IntoIterator<Item = BsaDirSource<D>>,
        W: Write + Seek {
            match self {
                ForSomeBsaVersion10X::V103(writer) => writer.write_bsa(dirs, out).map_err(ForSomeBsaVersion10X::V103),
                ForSomeBsaVersion10X::V104(writer) => writer.write_bsa(dirs, out).map_err(ForSomeBsaVersion10X::V104),
                ForSomeBsaVersion10X::V105(writer) => writer.write_bsa(dirs, out).map_err(ForSomeBsaVersion10X::V105),
            }
    }
}
