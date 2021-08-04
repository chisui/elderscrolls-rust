use std::io::{Read, Write, Seek, Result, Error};
use std::{error, fmt, str, mem};
use bytemuck::{Pod, Zeroable};

use super::bin::{self, err};


#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Zeroable, Pod)]
pub struct MagicNumber {
    pub value: [u8; 4],
}
const MGNR_V100: MagicNumber = MagicNumber{ value: [0,0,1,0] };
const MGNR_V10X: MagicNumber = MagicNumber{ value: *b"BSA\0" };
const MGNR_BTDX: MagicNumber = MagicNumber{ value: *b"BTDX" };
impl bin::Readable for MagicNumber {
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<MagicNumber> {
        bin::read_struct(reader)
    }
}
impl bin::Writable for MagicNumber {
    fn size(&self) -> usize { mem::size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        (&self.value as &[u8]).write_here(writer)
    }
}
impl fmt::Display for MagicNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MagicNumber{ value: [a, b, c, d] } = self;
        write!(f, "{:x}{:x}{:x}{:x}", a, b, c, d)
    }
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
#[derive(Debug)]
pub enum Unknown {
    Version(u8),
    MagicNumber(MagicNumber),
    VersionString(String),
}
impl error::Error for Unknown {}
impl fmt::Display for Unknown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unknown::Version(v)       => write!(f, "Unknown version {}", v),
            Unknown::VersionString(v) => write!(f, "Unknown version {}", v),
            Unknown::MagicNumber(n)   => write!(f, "Unknown magic number {}", n),
        }
    }
}

impl bin::Writable for Version {
    fn size(&self) -> usize { 
        match self {
            Version::V100 => mem::size_of::<MagicNumber>(),
            _ => mem::size_of::<MagicNumber>() + mem::size_of::<u32>(),
        }
     }
    fn write_here<W: Write>(&self, mut writer: W) -> Result<()> {
        match self {
            Version::V100 => MGNR_V100.write_here(writer),
            Version::V200(v) => {
                MGNR_BTDX.write_here(&mut writer)?;
                v.write_here(writer)
            }
            Version::V10X(v) => {
                MGNR_V10X.write_here(&mut writer)?;
                (*v as u32).write_here(writer)
            }
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
                    103 => Ok(Version10X::V103),
                    104 => Ok(Version10X::V104),
                    105 => Ok(Version10X::V105),
                    _   => err(Unknown::Version(version)),
                }.map(Version::V10X)
            },
            nr => err(Unknown::MagicNumber(nr))
        }
    }
}
impl str::FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "v100" | "tes3" | "morrowind" => Ok(Version::V100),
            "v103" | "tes4" | "oblivion"  => Ok(Version::V10X(Version10X::V103)),
            "v104" | "tes5" | "skyrim" | "f3" | "fallout3" | "fnv" | "newvegas" | "falloutnewvegas" => Ok(Version::V10X(Version10X::V104)),
            "v105" | "tes5se" | "skyrimse" => Ok(Version::V10X(Version10X::V105)),
            "v200" | "f4" | "fallout4" | "f76" | "fallout76" => Ok(Version::V200(1)),
            _ => err(Unknown::VersionString(String::from(s))),
        }
    }
}
