use std::mem::size_of;
use std::convert::TryFrom;
use std::io::{self, BufReader, Read, Seek, Write};
use std::path::Path;
use std::fs::File;
use std::fmt;

use thiserror::Error;
use num_enum::{TryFromPrimitive, IntoPrimitive};

use crate::read::Reader;
use crate::bin::{Fixed, Readable, ReadableFixed, VarSize, Writable, WritableFixed, concat_bytes};
use crate::v001::ReaderV001;
use crate::v103::ReaderV103;
use crate::v104::ReaderV104;
use crate::v105::ReaderV105;



#[derive(Debug, Error)]
#[error("Unknown magic number 0x{0:x}")]
pub struct UnknownMagicNumber(u32);

#[derive(Debug, Error)]
#[error("Unsupported Version {0}")]
pub struct UnsupportedVersion(pub Version);

#[derive(Debug, Error)]
#[error("Unknown version {0}")]
pub struct UnknownVersion(u32);


#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum MagicNumber {
    V001 = concat_bytes([0,0,1,0]),
    BSA0 = concat_bytes(*b"BSA\0"),
    BTDX = concat_bytes(*b"BTDX"),
    DX10 = concat_bytes(*b"DX10"),
}
impl Fixed for MagicNumber {
    fn pos() -> usize { 0 }
}
derive_var_size_via_size_of!(MagicNumber);
impl ReadableFixed for MagicNumber {
    fn read_fixed<R: Read + Seek>(mut reader: R) -> io::Result<MagicNumber> {
        Self::move_to_start(&mut reader)?;
        let raw = u32::read_bin(reader)?;
        MagicNumber::try_from(raw)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, UnknownMagicNumber(raw)))
    }
}
impl WritableFixed for MagicNumber {
    fn write_fixed<W: Write + Seek>(&self, mut writer: W) -> io::Result<()> {
        Self::move_to_start(&mut writer)?;
        u32::from(*self).write(writer)
    }
}
impl fmt::Display for MagicNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", *self as u32)
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum Version10X {
    V103 = 103,
    V104 = 104,
    V105 = 105,
}
derive_var_size_via_size_of!(Version10X);
impl Version10X {
    pub fn read_bsa<R: Read + Seek>(&self, reader: R) -> io::Result<crate::SomeReaderV10X<R>> {
        match self {
            Version10X::V103 => ReaderV103::read_bsa(reader).map(crate::SomeReaderV10X::V103),
            Version10X::V104 => ReaderV104::read_bsa(reader).map(crate::SomeReaderV10X::V104),
            Version10X::V105 => ReaderV105::read_bsa(reader).map(crate::SomeReaderV10X::V105),
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
        let raw = u32::read_bin(&mut reader)?;
        Version10X::try_from(raw)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, UnknownVersion(raw)))
    }
}
impl WritableFixed for Version10X {
    fn write_fixed<W: Write + Seek>(&self, mut writer: W) -> io::Result<()> {
        Self::move_to_start(&mut writer)?;
        u32::from(*self).write(writer)
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
    pub fn open<P>(&self, path: P) -> io::Result<crate::SomeReader<BufReader<File>>>
    where P: AsRef<Path> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        self.read_bsa(buf)
    }
    pub fn read_bsa<R: Read + Seek>(&self, reader: R) -> io::Result<crate::SomeReader<R>> {
        match self {
            Version::V001 => ReaderV001::read_bsa(reader).map(crate::SomeReader::V001),
            Version::V10X(v) => v.read_bsa(reader).map(crate::SomeReader::V10X),
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
            Version::V10X(v) => v.size(),
            Version::BA2(_, v) => v.size(),
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
