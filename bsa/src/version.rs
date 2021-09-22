use std::{fmt, str};
use std::io::{self, Read, Write, Seek, Result};
use std::mem::size_of;

use thiserror::Error;

use super::bin;
use super::magicnumber::MagicNumber;

#[derive(Debug, Error)]
pub enum Unknown {
    #[error("Unknown version {0}")]
    Version(u8),
    #[error("Unknown version {0}")]
    MagicNumber(u32),
    #[error("Unknown magic number {0}")]
    VersionString(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version {
    V100, // TES3
    V10X(Version10X),
    V200(u32), // F4 F76
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version10X {
    V103 = 103, // TES4
    V104 = 104, // F3, FNV, TES5
    V105 = 105, // TES5se
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::V100 => write!(f, "v100"),
            Version::V10X(Version10X::V103) => write!(f, "v103"),
            Version::V10X(Version10X::V104) => write!(f, "v104"),
            Version::V10X(Version10X::V105) => write!(f, "v105"),
            Version::V200(v) => write!(f, "BA2 v{:03}", v),
        }
    }
}

impl bin::Writable for Version {
    fn size(&self) -> usize { 
        size_of::<MagicNumber>() + match self {
            Version::V100 => 0,
            Version::V10X(_) => size_of::<Version10X>(),
            Version::V200(_) => size_of::<u32>(),
        }
     }
    fn write_here<W: Write>(&self, mut writer: W) -> Result<()> {
        match self {
            Version::V100 => MagicNumber::V100.write_here(writer),
            Version::V200(v) => {
                MagicNumber::BTDX.write_here(&mut writer)?;
                v.write_here(writer)
            }
            Version::V10X(v) => {
                MagicNumber::V10X.write_here(&mut writer)?;
                (*v as u32).write_here(writer)
            }
        }
    }
}
impl bin::Readable for Version {
    fn read_here<R: Read + Seek>(mut buffer: R, _: &()) -> Result<Self> {
        let mg_nr = MagicNumber::read(&mut buffer, &())?;
        match mg_nr {
            MagicNumber::V100 => Ok(Version::V100),
            MagicNumber::V10X => {
                let version: u8 = bin::read_struct(&mut buffer)?;
                match version {
                    103 => Ok(Version10X::V103),
                    104 => Ok(Version10X::V104),
                    105 => Ok(Version10X::V105),
                    _ => Err(io::Error::new(io::ErrorKind::InvalidData, Unknown::Version(version))),
                }.map(Version::V10X)
            },
            MagicNumber::BTDX => {
                let v = u32::read_here0(&mut buffer)?;
                Ok(Version::V200(v))
            }
        }
    }
}
impl str::FromStr for Version {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "v100" | "tes3" | "morrowind" => Ok(Version::V100),
            "v103" | "tes4" | "oblivion"  => Ok(Version::V10X(Version10X::V103)),
            "v104" | "tes5" | "skyrim" | "f3" | "fallout3" | "fnv" | "newvegas" | "falloutnewvegas" => Ok(Version::V10X(Version10X::V104)),
            "v105" | "tes5se" | "skyrimse" => Ok(Version::V10X(Version10X::V105)),
            "v200" | "f4" | "fallout4" | "f76" | "fallout76" => Ok(Version::V200(1)),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, Unknown::VersionString(String::from(s)))),
        }
    }
}
