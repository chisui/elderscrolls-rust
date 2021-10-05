#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization, arbitrary_enum_discriminant)]
pub mod hash;
pub mod read;
pub mod write;
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
    io::{BufReader, Read, Seek, Write, Result},
    fmt,
    fs::File,
    path::Path,
};

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

pub enum SomeBsaHeader {
    V001(v001::Header),
    V103(v103::Header),
    V104(v104::Header),
    V105(v105::Header),
}
impl fmt::Display for SomeBsaHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SomeBsaHeader::V001(header) => header.fmt(f),
            SomeBsaHeader::V103(header) => header.fmt(f),
            SomeBsaHeader::V104(header) => header.fmt(f),
            SomeBsaHeader::V105(header) => header.fmt(f),
        }
    }
}

pub enum SomeBsaReader<R> {
    V001(v001::BsaReader<R>),
    V103(v103::BsaReader<R>),
    V104(v104::BsaReader<R>),
    V105(v105::BsaReader<R>),
}
impl<R> SomeBsaReader<R> {

    pub fn version(&self) -> Version {
        match self {
            SomeBsaReader::V001(_) => Version::V001,
            SomeBsaReader::V103(_) => Version::V10X(Version10X::V103),
            SomeBsaReader::V104(_) => Version::V10X(Version10X::V104),
            SomeBsaReader::V105(_) => Version::V10X(Version10X::V105),
        }
    }
}

pub fn open<P>(path: P) -> Result<SomeBsaReader<BufReader<File>>>
where P: AsRef<Path> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    read(buf)
}
pub fn read<R>(mut reader: R) -> Result<SomeBsaReader<R>>
where R: Read + Seek {
    let v = <Version as Readable>::read(&mut reader, &())?;
    v.read(reader)
}

impl<R: Read + Seek> BsaReader for SomeBsaReader<R> {
    type Header = SomeBsaHeader;

    fn header(&self) -> Self::Header {
        match self {
            SomeBsaReader::V001(bsa) => SomeBsaHeader::V001(bsa.header()),
            SomeBsaReader::V103(bsa) => SomeBsaHeader::V103(bsa.header()),
            SomeBsaReader::V104(bsa) => SomeBsaHeader::V104(bsa.header()),
            SomeBsaReader::V105(bsa) => SomeBsaHeader::V105(bsa.header()),
        }
    }

    fn dirs(&mut self) -> Result<Vec<BsaDir>> {
        match self {
            SomeBsaReader::V001(bsa) => bsa.dirs(),
            SomeBsaReader::V103(bsa) => bsa.dirs(),
            SomeBsaReader::V104(bsa) => bsa.dirs(),
            SomeBsaReader::V105(bsa) => bsa.dirs(),
        }
    }

    fn extract<W: Write>(&mut self, file: &BsaFile, writer: W) -> Result<()> {
        match self {
            SomeBsaReader::V001(bsa) => bsa.extract(file, writer),
            SomeBsaReader::V103(bsa) => bsa.extract(file, writer),
            SomeBsaReader::V104(bsa) => bsa.extract(file, writer),
            SomeBsaReader::V105(bsa) => bsa.extract(file, writer),
        }
    }
}
