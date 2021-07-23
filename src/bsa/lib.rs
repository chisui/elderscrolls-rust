#![feature(associated_type_defaults, wrapping_int_impl)]
pub mod hash;
pub mod archive;
pub mod bin;
pub mod bzstring;
pub mod version;
pub mod v103;
pub mod v104;
pub mod v105;

use v103::Has;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, Result, Error, ErrorKind};
use std::fmt;
use archive::{BsaDir, BsaFile};
use bin::Readable;
use version::Version;


pub enum Bsa {
    V103(v103::Header),
    V104(v104::Header), 
    V105(v105::Header),
}
impl Bsa {
    pub fn open<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let version = Version::read(&mut reader, &())?;
        match version {
            Version::V103 => {
                let header = v103::Header::read(&mut reader, &())?;
                Ok(Bsa::V103(header))
            }
            Version::V104 => {
                let header = v104::Header::read(&mut reader, &())?;
                Ok(Bsa::V104(header))
            }
            Version::V105 => {
                let header = v105::Header::read(&mut reader, &())?;
                Ok(Bsa::V105(header))
            }
            v => Err(Error::new(ErrorKind::InvalidData, format!("Unsupported version {}", v)))
        }
    }

    pub fn read_dirs<R: Read + Seek>(&self, mut reader: R) -> Result<Vec<BsaDir>> {
        match self {
            Bsa::V105(header) => v105::file_tree(&mut reader, header),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unsupported version")),
        }
    }

    pub fn extract<R: Read + Seek>(&self, file: BsaFile, out_path: &Path, reader: R) -> Result<()> {
        let out_file = File::create(out_path)?;
        match self {
            Bsa::V103(header) => v103::extract(!header.has(v103::ArchiveFlag::IncludeFileNames), file, reader, out_file),
            Bsa::V104(header) => v104::extract(!header.has(v104::ArchiveFlag::IncludeFileNames), file, reader, out_file),
            Bsa::V105(header) => v105::extract(!header.has(v105::ArchiveFlag::IncludeFileNames), file, reader, out_file),
        }
    }
}

impl fmt::Display for Bsa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bsa::V103(header) => {
                writeln!(f, "BSA v103 file, format used by: TES IV: Oblivion")?;
                writeln!(f, "{}", header)
            },
            Bsa::V104(header) => {
                writeln!(f, "BSA v104 file, format used by: Fallout 3, Fallout: NV, TES V: Skyrim")?;
                writeln!(f, "{}", header)
            },
            Bsa::V105(header) => {
                writeln!(f, "BSA v105 file, format used by: TES V: Skyrim Special Edition")?;
                writeln!(f, "{}", header)
            },
        }
    }
}
