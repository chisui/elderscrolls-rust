#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization, arbitrary_enum_discriminant)]
pub mod hash;
pub mod archive;
pub mod bin;
pub mod bzstring;
pub mod version;
pub mod v001;
pub mod v10x;
pub mod v103;
pub mod v104;
pub mod v105;

use std::io::{Read, Seek, Write, Result};
use std::{error, fmt};
use archive::{Bsa, BsaDir, BsaFile};
use bin::{err, Readable};
use version::{Version, Version10X};


#[derive(Debug)]
struct UnsupportedVersion(pub Version);
impl error::Error for UnsupportedVersion {}
impl fmt::Display for UnsupportedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsupported Version {}", self.0)
    }
}
pub enum SomeBsa {
    V103(v103::V103),
    V104(v104::V104),
    V105(v105::V105),
}
impl fmt::Display for SomeBsa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SomeBsa::V103(bsa) => bsa.fmt(f),
            SomeBsa::V104(bsa) => bsa.fmt(f),
            SomeBsa::V105(bsa) => bsa.fmt(f),
        }
    }
}
impl Bsa for SomeBsa {
    fn open<R: Read + Seek>(mut reader: R) -> Result<SomeBsa> {
        let version = Version::read(&mut reader, &())?;
        match version {
            Version::V10X(v) => match v {
                Version10X::V103 => {
                    let bsa = v103::V103::open(&mut reader)?;
                    Ok(SomeBsa::V103(bsa))
                },
                Version10X::V104 => {
                    let bsa = v104::V104::open(&mut reader)?;
                    Ok(SomeBsa::V104(bsa))
                },
                Version10X::V105 => {
                    let bsa = v105::V105::open(&mut reader)?;
                    Ok(SomeBsa::V105(bsa))
                },
            },
            v => err(UnsupportedVersion(v)),
        }
    }

    fn version(&self) -> Version {
        match self {
            SomeBsa::V103(bsa) => bsa.version(),
            SomeBsa::V104(bsa) => bsa.version(),
            SomeBsa::V105(bsa) => bsa.version(),
        }
    }

    fn read_dirs<R: Read + Seek>(&self, reader: R) -> Result<Vec<BsaDir>> {
        match self {
            SomeBsa::V103(bsa) => bsa.read_dirs(reader),
            SomeBsa::V104(bsa) => bsa.read_dirs(reader),
            SomeBsa::V105(bsa) => bsa.read_dirs(reader),
        }
    }

    fn extract<R: Read + Seek, W: Write>(&self, file: BsaFile, reader: R, writer: W) -> Result<()> {
        match self {
            SomeBsa::V103(bsa) => bsa.extract(file, reader, writer),
            SomeBsa::V104(bsa) => bsa.extract(file, reader, writer),
            SomeBsa::V105(bsa) => bsa.extract(file, reader, writer),
        }
    }
}