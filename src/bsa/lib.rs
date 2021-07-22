#![feature(once_cell, associated_type_defaults, wrapping_int_impl)]
use std::io::{Read, Seek, Result, Error, ErrorKind};

pub mod hash;
pub mod archive;
pub mod bin;
pub mod bzstring;
pub mod version;
pub mod v103;
pub mod v104;
pub mod v105;

use archive::BsaDir;
use bin::Readable;
use version::Version;


pub enum Bsa {
    V103(v103::Header),
    V104(v104::Header), 
    V105(v105::Header),
}
impl Bsa {
    pub fn open<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let version = Version::read(&mut reader, ())?;
        match version {
            Version::V103 => {
                let header = v103::Header::read(&mut reader, ())?;
                Ok(Bsa::V103(header))
            }
            Version::V104 => {
                let header = v104::Header::read(&mut reader, ())?;
                Ok(Bsa::V104(header))
            }
            Version::V105 => {
                let header = v105::Header::read(&mut reader, ())?;
                Ok(Bsa::V105(header))
            }
            v => Err(Error::new(ErrorKind::InvalidData, format!("Unsupported version {}", v)))
        }
    }

    pub fn read_dirs<R: Read + Seek>(self, mut reader: R) -> Result<Vec<BsaDir>> {
        match self {
            Bsa::V105(header) => v105::file_tree(&mut reader, header),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unsupported version")),
        }
    }
}
