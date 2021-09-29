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

use std::io::{Read, Seek, Write, Result, Error, ErrorKind};
use std::fmt;
use thiserror::Error;
use archive::{Bsa, BsaDir, BsaFile};
use bin::Readable;
use version::{Version, Version10X};


#[derive(Debug, Error)]
#[error("Unsupported Version {0}")]
struct UnsupportedVersion(pub Version);


pub enum BsaArchive<R> {
    V103(v103::BsaArchive<R>),
    V104(v104::BsaArchive<R>),
    V105(v105::BsaArchive<R>),
}
pub enum BsaHeader {
    V103(v103::Header),
    V104(v104::Header),
    V105(v105::Header),
}
impl<R> fmt::Display for BsaArchive<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BsaArchive::V103(bsa) => bsa.fmt(f),
            BsaArchive::V104(bsa) => bsa.fmt(f),
            BsaArchive::V105(bsa) => bsa.fmt(f),
        }
    }
}
impl<R: Read + Seek> BsaArchive<R> {
    pub fn open(mut reader: R) -> Result<BsaArchive<R>> {
        match Version::read(&mut reader, &())? {
            Version::V10X(v) => match v {
                Version10X::V103 => v103::BsaArchive::open(reader)
                    .map(BsaArchive::V103),
                Version10X::V104 => v104::BsaArchive::open(reader)
                    .map(BsaArchive::V104),
                Version10X::V105 => v105::BsaArchive::open(reader)
                    .map(BsaArchive::V105),
            },
            v => Err(Error::new(ErrorKind::InvalidData, UnsupportedVersion(v))),
        }
    }
}
impl<R: Read + Seek> Bsa for BsaArchive<R> {
    type Header = BsaHeader;

    fn version(&self) -> Version {
        match self {
            BsaArchive::V103(bsa) => bsa.version(),
            BsaArchive::V104(bsa) => bsa.version(),
            BsaArchive::V105(bsa) => bsa.version(),
        }
    }

    fn header(&self) -> Self::Header {
        match self {
            BsaArchive::V103(bsa) => BsaHeader::V103(bsa.header()),
            BsaArchive::V104(bsa) => BsaHeader::V104(bsa.header()),
            BsaArchive::V105(bsa) => BsaHeader::V105(bsa.header()),
        }
    }

    fn read_dirs(&mut self) -> Result<Vec<BsaDir>> {
        match self {
            BsaArchive::V103(bsa) => bsa.read_dirs(),
            BsaArchive::V104(bsa) => bsa.read_dirs(),
            BsaArchive::V105(bsa) => bsa.read_dirs(),
        }
    }

    fn extract<W: Write>(&mut self, file: BsaFile, writer: W) -> Result<()> {
        match self {
            BsaArchive::V103(bsa) => bsa.extract(file, writer),
            BsaArchive::V104(bsa) => bsa.extract(file, writer),
            BsaArchive::V105(bsa) => bsa.extract(file, writer),
        }
    }
}
