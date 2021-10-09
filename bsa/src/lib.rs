#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization)]
#[macro_use]
mod bin;
mod compress;
mod str;
pub mod read;
pub mod write;
mod hash;
pub mod version;
pub mod v001;
mod v10x;
pub mod v103;
pub mod v104;
pub mod v105;

use std::io::{self, Read, Seek, Write};
use bin::ReadableFixed;
use thiserror::Error;

pub use crate::hash::Hash;
pub use crate::version::*;
pub use crate::bin::DataSource;
pub use crate::read::{open, Reader, EntryId};
pub use crate::write::{list_dir, Writer};
pub use crate::v001::{V001, ReaderV001, HeaderV001, WriterV001};
pub use crate::v10x::{ToArchiveBitFlags, FileFlag};
pub use crate::v103::{V103, ReaderV103, HeaderV103, WriterV103, ArchiveFlagV103};
pub use crate::v104::{V104, ReaderV104, HeaderV104, WriterV104, ArchiveFlagV104};
pub use crate::v105::{V105, ReaderV105, HeaderV105, WriterV105, ArchiveFlagV105};


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum ForSomeVersion<A001, A10X> {
    #[error("{0}")] V001(A001),
    #[error("{0}")] V10X(A10X),
}
impl<A001, A103, A104, A105> ForSomeVersion<A001, ForSomeVersion10X<A103, A104, A105>> {
    pub fn version(&self) -> Version {
        match self {
            ForSomeVersion::V001(_) => Version::V001,
            ForSomeVersion::V10X(v) => Version::V10X(v.version()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum ForSomeVersion10X<A103, A104, A105> {
    #[error("{0}")] V103(A103),
    #[error("{0}")] V104(A104),
    #[error("{0}")] V105(A105),
}
impl<A103, A104, A105> ForSomeVersion10X<A103, A104, A105> {
    pub fn version(&self) -> Version10X {
        match self {
            ForSomeVersion10X::V103(_) => Version10X::V103,
            ForSomeVersion10X::V104(_) => Version10X::V104,
            ForSomeVersion10X::V105(_) => Version10X::V105,
        }
    }
}

pub type SomeHeaderV10X = ForSomeVersion10X<HeaderV103, HeaderV104, HeaderV105>;
pub type SomeHeader = ForSomeVersion<HeaderV001, SomeHeaderV10X>;

pub type SomeReaderV10X<R> = ForSomeVersion10X<ReaderV103<R>, ReaderV104<R>, ReaderV105<R>>;
pub type SomeReader<R> = ForSomeVersion<ReaderV001<R>, SomeReaderV10X<R>>;

pub type SomeWriterV10X = ForSomeVersion10X<WriterV103, WriterV104, WriterV105>;
pub type SomeWriter = ForSomeVersion<WriterV001, SomeWriterV10X>;

pub type SomeRoot = ForSomeVersion<Vec<read::File>, Vec<read::Dir>>;

impl<R> Reader for SomeReader<R>
where R: Read + Seek {
    type Header = SomeHeader;
    type Root = SomeRoot;
    type In = R;

    fn read_bsa(mut reader: R) -> io::Result<Self> {
        Version::read_fixed(&mut reader)?
            .read_bsa(reader)
    }

    fn header(&self) -> Self::Header {
        match self {
            ForSomeVersion::V001(bsa) => ForSomeVersion::V001(bsa.header()),
            ForSomeVersion::V10X(bsa) => ForSomeVersion::V10X(bsa.header()),
        }
    }

    fn list(&mut self) -> io::Result<SomeRoot> {
        match self {
            ForSomeVersion::V001(bsa) => bsa.list().map(SomeRoot::V001),
            ForSomeVersion::V10X(bsa) => bsa.list().map(SomeRoot::V10X),
        }
    }

    fn extract<W: Write>(&mut self, file: &read::File, writer: W) -> io::Result<()> {
        match self {
            ForSomeVersion::V001(bsa) => bsa.extract(file, writer),
            ForSomeVersion::V10X(bsa) => bsa.extract(file, writer),
        }
    }
}


impl<R> Reader for SomeReaderV10X<R>
where R: Read + Seek {
    type Header = SomeHeaderV10X;
    type Root = Vec<read::Dir>;
    type In = R;

    fn read_bsa(mut reader: R) -> io::Result<Self> {
        Version10X::read_fixed(&mut reader)?
            .read_bsa(reader)
    }

    fn header(&self) -> Self::Header {
        match self {
            ForSomeVersion10X::V103(bsa) => ForSomeVersion10X::V103(bsa.header()),
            ForSomeVersion10X::V104(bsa) => ForSomeVersion10X::V104(bsa.header()),
            ForSomeVersion10X::V105(bsa) => ForSomeVersion10X::V105(bsa.header()),
        }
    }

    fn list(&mut self) -> io::Result<Vec<read::Dir>> {
        match self {
            ForSomeVersion10X::V103(bsa) => bsa.list(),
            ForSomeVersion10X::V104(bsa) => bsa.list(),
            ForSomeVersion10X::V105(bsa) => bsa.list(),
        }
    }

    fn extract<W: Write>(&mut self, file: &read::File, writer: W) -> io::Result<()> {
        match self {
            ForSomeVersion10X::V103(bsa) => bsa.extract(file, writer),
            ForSomeVersion10X::V104(bsa) => bsa.extract(file, writer),
            ForSomeVersion10X::V105(bsa) => bsa.extract(file, writer),
        }
    }
}

impl Writer for SomeWriter {
    type Err = ForSomeVersion<
        <WriterV001 as Writer>::Err,
        ForSomeVersion10X<
            <WriterV103 as Writer>::Err,
            <WriterV104 as Writer>::Err,
            <WriterV105 as Writer>::Err,
        >,
    >;

    fn write_bsa<DS, D, W>(&self, dirs: DS, out: W) -> Result<(), Self::Err>
    where
        D: bin::DataSource,
        DS: IntoIterator<Item = write::Dir<D>>,
        W: Write + Seek {
            match self {
                ForSomeVersion::V001(writer) => writer.write_bsa(dirs, out).map_err(ForSomeVersion::V001),
                ForSomeVersion::V10X(writer) => writer.write_bsa(dirs, out).map_err(ForSomeVersion::V10X),
            }
    }
}


impl Writer for SomeWriterV10X {
    type Err = ForSomeVersion10X<
        <WriterV103 as Writer>::Err,
        <WriterV104 as Writer>::Err,
        <WriterV105 as Writer>::Err,
    >;

    fn write_bsa<DS, D, W>(&self, dirs: DS, out: W) -> Result<(), Self::Err>
    where
        D: bin::DataSource,
        DS: IntoIterator<Item = write::Dir<D>>,
        W: Write + Seek {
            match self {
                ForSomeVersion10X::V103(writer) => writer.write_bsa(dirs, out).map_err(ForSomeVersion10X::V103),
                ForSomeVersion10X::V104(writer) => writer.write_bsa(dirs, out).map_err(ForSomeVersion10X::V104),
                ForSomeVersion10X::V105(writer) => writer.write_bsa(dirs, out).map_err(ForSomeVersion10X::V105),
            }
    }
}
