#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization, arbitrary_enum_discriminant)]
pub mod hash;
pub mod archive;
pub mod bin;
pub mod str;
pub mod magicnumber;
pub mod version;
pub mod v001;
pub mod v10x;
pub mod v103;
pub mod v104;
pub mod v105;

use std::{
    io::{Read, Seek, Write, Result, Error, ErrorKind},
    fmt,
};
use thiserror::Error;

use archive::{BsaDir, BsaFile};
use bin::Readable;

pub use version::{Version, Version10X};
pub use {
    v103::V103,
    v104::V104,
    v105::V105,
};

#[derive(Debug, Error)]
#[error("Unsupported Version {0}")]
struct UnsupportedVersion(pub Version);


pub enum SomeBsaHeader {
    V103(v103::Header),
    V104(v104::Header),
    V105(v105::Header),
}
impl fmt::Display for SomeBsaHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SomeBsaHeader::V103(header) => header.fmt(f),
            SomeBsaHeader::V104(header) => header.fmt(f),
            SomeBsaHeader::V105(header) => header.fmt(f),
        }
    }
}

pub enum SomeBsaReader<R> {
    V103(v103::BsaReader<R>),
    V104(v104::BsaReader<R>),
    V105(v105::BsaReader<R>),
}
impl<R> fmt::Display for SomeBsaReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SomeBsaReader::V103(bsa) => bsa.fmt(f),
            SomeBsaReader::V104(bsa) => bsa.fmt(f),
            SomeBsaReader::V105(bsa) => bsa.fmt(f),
        }
    }
}
impl<R: Read + Seek> SomeBsaReader<R> {
    pub fn open(mut reader: R) -> Result<SomeBsaReader<R>> {
        match Version::read(&mut reader, &())? {
            Version::V10X(v) => match v {
                Version10X::V103 => v103::BsaReader::open(reader)
                    .map(SomeBsaReader::V103),
                Version10X::V104 => v104::BsaReader::open(reader)
                    .map(SomeBsaReader::V104),
                Version10X::V105 => v105::BsaReader::open(reader)
                    .map(SomeBsaReader::V105),
            },
            v => Err(Error::new(ErrorKind::InvalidData, UnsupportedVersion(v))),
        }
    }
}
impl<R: Read + Seek> archive::BsaReader for SomeBsaReader<R> {
    type Header = SomeBsaHeader;

    fn version(&self) -> Version {
        match self {
            SomeBsaReader::V103(bsa) => bsa.version(),
            SomeBsaReader::V104(bsa) => bsa.version(),
            SomeBsaReader::V105(bsa) => bsa.version(),
        }
    }

    fn header(&self) -> Self::Header {
        match self {
            SomeBsaReader::V103(bsa) => SomeBsaHeader::V103(bsa.header()),
            SomeBsaReader::V104(bsa) => SomeBsaHeader::V104(bsa.header()),
            SomeBsaReader::V105(bsa) => SomeBsaHeader::V105(bsa.header()),
        }
    }

    fn read_dirs(&mut self) -> Result<Vec<BsaDir>> {
        match self {
            SomeBsaReader::V103(bsa) => bsa.read_dirs(),
            SomeBsaReader::V104(bsa) => bsa.read_dirs(),
            SomeBsaReader::V105(bsa) => bsa.read_dirs(),
        }
    }

    fn extract<W: Write>(&mut self, file: &BsaFile, writer: W) -> Result<()> {
        match self {
            SomeBsaReader::V103(bsa) => bsa.extract(file, writer),
            SomeBsaReader::V104(bsa) => bsa.extract(file, writer),
            SomeBsaReader::V105(bsa) => bsa.extract(file, writer),
        }
    }
}
