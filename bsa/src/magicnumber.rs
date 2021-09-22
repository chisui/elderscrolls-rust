use std::fmt;
use std::io::{self, Read, Write, Seek};
use std::convert::TryFrom;
use std::mem::size_of;

use thiserror::Error;

use super::bin;


#[derive(Debug, Error)]
#[error("Unknown magic number {0:#}")]
pub struct UnknownMagicNumber(pub u32);
impl From<UnknownMagicNumber> for io::Error {
    fn from(err: UnknownMagicNumber) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, err)
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MagicNumber {
    V100 = bin::concat_bytes([0,0,1,0]),
    V10X = bin::concat_bytes(*b"BSA\0"),
    BTDX = bin::concat_bytes(*b"BTDX"),
}
impl From<MagicNumber> for u32 {
    fn from(nr: MagicNumber) -> u32 {
        nr as u32
    }
}
impl TryFrom<u32> for MagicNumber {
    type Error = UnknownMagicNumber;
    fn try_from(i: u32) -> Result<Self, UnknownMagicNumber> {
        if i == MagicNumber::V100 as u32 { Ok(MagicNumber::V100) }
        else if i == MagicNumber::V10X as u32 { Ok(MagicNumber::V10X) }
        else if i == MagicNumber::BTDX as u32 { Ok(MagicNumber::BTDX) }
        else { Err(UnknownMagicNumber(i)) }
    }
}
impl bin::Readable for MagicNumber {
    fn offset(_: &()) -> Option<usize> { Some(0) }
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> io::Result<MagicNumber> {
        let nr = u32::read_here0(reader)?;
        MagicNumber::try_from(nr)
            .map_err(io::Error::from)
    }
}
impl bin::Writable for MagicNumber {
    fn size(&self) -> usize { size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> io::Result<()> {
        (*self as u32).write_here(writer)
    }
}
impl fmt::Display for MagicNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", *self as u32)
    }
}
