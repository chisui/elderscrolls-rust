use std::io::{Read, Seek, Result};
use std::{error, fmt};
use bytemuck::{Pod, Zeroable};

use super::bin::{self, err};


#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Zeroable, Pod)]
pub struct MagicNumber {
    pub value: [u8; 4],
}
const MGNR_V100: MagicNumber = MagicNumber{ value: [0,0,1,0] };
const MGNR_V10X: MagicNumber = MagicNumber{ value: *b"BSA\0" };
impl bin::Readable for MagicNumber {
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<MagicNumber> {
        bin::read_struct(reader)
    }
}
impl fmt::Display for MagicNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MagicNumber{ value: [a, b, c, d] } = self;
        write!(f, "{:x}{:x}{:x}{:x}", a, b, c, d)
    }
}


#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
pub enum Version {
    V100, // TES3
    V103, // TES4
    V104, // F3, FNV, TES5
    V105, // TES5se
    V200, // F4 F76
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Version::V100 => "v100",
            Version::V103 => "v103",
            Version::V104 => "v104",
            Version::V105 => "v105",
            Version::V200 => "v200",
        })
    }
}
#[derive(Debug)]
pub enum Unknown {
    Version(u8),
    MagicNumber(MagicNumber),
}
impl error::Error for Unknown {}
impl fmt::Display for Unknown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unknown::Version(v)     => write!(f, "Unknown version {}", v),
            Unknown::MagicNumber(n) => write!(f, "Unknown magic number {}", n),
        }
    }
}
impl bin::Readable for Version {
    fn read_here<R: Read + Seek>(mut buffer: R, _: &()) -> Result<Self> {
        let mg_nr = MagicNumber::read(&mut buffer, &())?;
        match mg_nr {
            MGNR_V100 => Ok(Version::V100),
            MGNR_V10X => {
                let version: u8 = bin::read_struct(&mut buffer)?;
                match version {
                    103 => Ok(Version::V103),
                    104 => Ok(Version::V104),
                    105 => Ok(Version::V105),
                    _   => err(Unknown::Version(version)),
                }
            },
            nr => err(Unknown::MagicNumber(nr))
        }
    }
}
