use std::mem::size_of;
use std::convert::TryFrom;
use std::io::{self, BufReader, Read, Seek, Write};
use std::path::Path;
use std::fs::File;
use std::fmt;

use thiserror::Error;

use crate::read::BsaReader;
use crate::bin::{Fixed, Readable, ReadableFixed, VarSize, Writable, WritableFixed, concat_bytes};



#[derive(Debug, Error)]
#[error("Unknown magic number {0:#}")]
pub enum MagicNumberError {
    Unknown(u32),
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MagicNumber {
    V001 = concat_bytes([0,0,1,0]),
    BSA0 = concat_bytes(*b"BSA\0"),
    BTDX = concat_bytes(*b"BTDX"),
    DX10 = concat_bytes(*b"DX10"),
}
impl From<MagicNumber> for u32 {
    fn from(nr: MagicNumber) -> u32 {
        nr as u32
    }
}
impl TryFrom<u32> for MagicNumber {
    type Error = MagicNumberError;
    fn try_from(i: u32) -> Result<Self, MagicNumberError> {
        if i == MagicNumber::V001 as u32 { Ok(MagicNumber::V001) }
        else if i == MagicNumber::BSA0 as u32 { Ok(MagicNumber::BSA0) }
        else if i == MagicNumber::BTDX as u32 { Ok(MagicNumber::BTDX) }
        else if i == MagicNumber::DX10 as u32 { Ok(MagicNumber::DX10) }
        else { Err(MagicNumberError::Unknown(i)) }
    }
}
impl Fixed for MagicNumber {
    fn pos() -> usize { 0 }
}
impl ReadableFixed for MagicNumber {
    fn read_fixed<R: Read + Seek>(mut reader: R) -> io::Result<MagicNumber> {
        Self::move_to_start(&mut reader)?;
        let raw = u32::read_bin(reader)?;
        MagicNumber::try_from(raw)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }
}
impl WritableFixed for MagicNumber {
    fn write_fixed<W: Write + Seek>(&self, mut writer: W) -> io::Result<()> {
        Self::move_to_start(&mut writer)?;
        (*self as u32).write(writer)
    }
}
impl fmt::Display for MagicNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", *self as u32)
    }
}


#[derive(Debug, Error)]
#[error("Unsupported Version {0}")]
struct UnsupportedVersion(pub Version);

#[derive(Debug, Error)]
pub enum Unknown {
    #[error("Unknown magic number {0}")]
    MagicNumber(u32),
    #[error("Unknown version {0}")]
    Version(u32),
}


#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version10X {
    V103 = 103,
    V104 = 104,
    V105 = 105,
}
derive_var_size_via_size_of!(Version10X);
impl Version10X {
    pub fn read<R: Read + Seek>(&self, reader: R) -> io::Result<crate::SomeBsaReader<R>> {
        match self {
            Version10X::V103 => crate::v103::BsaReader::read_bsa(reader).map(crate::SomeBsaReader::V103),
            Version10X::V104 => crate::v104::BsaReader::read_bsa(reader).map(crate::SomeBsaReader::V104),
            Version10X::V105 => crate::v105::BsaReader::read_bsa(reader).map(crate::SomeBsaReader::V105),
        }
    }
}
impl fmt::Display for Version10X {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version10X::V103 => write!(f, "v103"),
            Version10X::V104 => write!(f, "v104"),
            Version10X::V105 => write!(f, "v105"),
        }
    }
}
impl Fixed for Version10X {
    fn pos() -> usize { size_of::<MagicNumber>() }
}
impl ReadableFixed for Version10X {
    fn read_fixed<R: Read + Seek>(mut reader: R) -> io::Result<Self> {
        Self::move_to_start(&mut reader)?;
        Ok(match u32::read_bin(&mut reader)? {
            103 => Version10X::V103,
            104 => Version10X::V104,
            105 => Version10X::V105,
            v => return Err(io::Error::new(io::ErrorKind::InvalidData, Unknown::Version(v))),
        })
    }
}
impl WritableFixed for Version10X {
    fn write_fixed<W: Write + Seek>(&self, mut writer: W) -> io::Result<()> {
        Self::move_to_start(&mut writer)?;
        (*self as u32).write(writer)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BA2Type {
    BTDX,
    DX10,
}
impl fmt::Display for BA2Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BA2Type::BTDX => write!(f, "BTDX"),
            BA2Type::DX10 => write!(f, "DX10"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version {
    V001,
    V10X(Version10X),
    BA2(BA2Type, u32),
}
impl Version {
    pub fn open<P>(&self, path: P) -> io::Result<crate::SomeBsaReader<BufReader<File>>>
    where P: AsRef<Path> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        self.read(buf)
    }
    pub fn read<R: Read + Seek>(&self, reader: R) -> io::Result<crate::SomeBsaReader<R>> {
        match self {
            Version::V001 => crate::v001::BsaReader::read_bsa(reader).map(crate::SomeBsaReader::V001),
            Version::V10X(v) => v.read(reader),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, UnsupportedVersion(*self))),
        }
    }
}
impl From<&Version> for MagicNumber {
    fn from(version: &Version) -> MagicNumber {
        match version {
            Version::V001    => MagicNumber::V001,
            Version::V10X(_) => MagicNumber::BSA0,
            Version::BA2(BA2Type::BTDX, _) => MagicNumber::BTDX,
            Version::BA2(BA2Type::DX10, _) => MagicNumber::DX10,
        }
    }
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::V001 => write!(f, "v100"),
            Version::V10X(v) => v.fmt(f),
            Version::BA2(t, v) => write!(f, "BA2 {} v{:03}", t, v),
        }
    }
}
impl VarSize for Version {
    fn size(&self) -> usize { 
        size_of::<MagicNumber>() + match self {
            Version::V001 => 0,
            Version::V10X(v) => (*v).size(),
            Version::BA2(_, _) => size_of::<u32>(),
        }
    }
}
impl Fixed for Version {
    fn pos() -> usize { 0 }
}
impl WritableFixed for Version {
    fn write_fixed<W: Write + Seek>(&self, mut writer: W) -> io::Result<()> {
        MagicNumber::from(self).write_fixed(&mut writer)?;
        match self {
            Version::V001 => Ok(()),
            Version::BA2(_, v) => v.write(writer),
            Version::V10X(v) => (*v as u32).write(writer),
        }
    }
}
impl ReadableFixed for Version {
    fn read_fixed<R: Read + Seek>(mut buffer: R) -> io::Result<Self> {
        Ok(match MagicNumber::read_fixed(&mut buffer)? {
            MagicNumber::V001 => Version::V001,
            MagicNumber::BSA0 => Version::V10X(Version10X::read_fixed(buffer)?),
            MagicNumber::BTDX => Version::BA2(BA2Type::BTDX, u32::read_bin(&mut buffer)?),
            MagicNumber::DX10 => Version::BA2(BA2Type::DX10, u32::read_bin(&mut buffer)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::bin::test::*;
    use super::*;

    #[test]
    fn write_read_identity_magic_number() {
        for v in [
            MagicNumber::V001,
            MagicNumber::BSA0,
            MagicNumber::BTDX,
        ] {
            write_read_fixed_identity(v)
        }
    }

    #[test]
    fn write_read_identity_version10x() {
        for v in [
            Version10X::V103,
            Version10X::V104,
            Version10X::V105,
        ] {
            write_read_fixed_identity(v)
        }
    }

    #[test]
    fn write_read_identity_version() {
        for v in [
            Version::V001, 
            Version::V10X(Version10X::V103),
            Version::V10X(Version10X::V104),
            Version::V10X(Version10X::V105),
            Version::BA2(BA2Type::BTDX, 12),
            Version::BA2(BA2Type::DX10, 42),
        ] {
            write_read_fixed_identity(v)
        }
    }
}
