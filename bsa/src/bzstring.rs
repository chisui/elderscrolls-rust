use std::io::{Read, Write, Seek, SeekFrom, Result, Error};
use std::str::{self, FromStr};
use std::convert::TryFrom;
use std::fmt;

use super::bin::{err, read_struct, Readable, Writable, write_many};


#[derive(Clone, PartialEq, Eq)]
pub struct BZString {
    pub value: String
}
impl fmt::Debug for BZString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}
impl From<BZString> for String {
    fn from(s: BZString) -> String {
        s.value
    }
}
impl TryFrom<Vec<u8>> for BZString {
    type Error = Error;
    fn try_from(chars: Vec<u8>) -> Result<BZString> {
        match str::from_utf8(&chars) {
            Ok(s) => BZString::from_str(s),
            Err(e) => err(e),
        }
    }
}
impl FromStr for BZString {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(BZString {
            value: s.to_owned()
        })
    }
}
impl Readable for BZString {
    type ReadableArgs = ();
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<BZString> {
        let length: u8 = read_struct(&mut reader)?;
        let mut chars: Vec<u8> = vec![0u8; (length - 1) as usize]; // length field includes null.
        reader.read_exact(&mut chars)?;
        reader.seek(SeekFrom::Current(1))?; // skip null byte.
        BZString::try_from(chars)
    }
}

#[derive(Debug)]
pub struct NullTerminated(pub BZString);
impl From<NullTerminated> for BZString {
    fn from(s: NullTerminated) -> BZString {
        s.0
    }
}
impl From<&NullTerminated> for BZString {
    fn from(s: &NullTerminated) -> BZString {
        s.0.clone()
    }
}
impl FromStr for NullTerminated {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let bzs = BZString::from_str(s)?;
        Ok(NullTerminated(bzs))
    }
}
impl Readable for NullTerminated {
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<Self> {
        let mut chars: Vec<u8> = Vec::with_capacity(32);
        loop {
            let c: u8 = read_struct(&mut reader)?;
            if c == 0 {
                break;
            }
            chars.push(c);
        }
        let s = BZString::try_from(chars)?;
        Ok(NullTerminated(s))
    }
}
impl Writable for NullTerminated {
    fn size(&self) -> usize {
        self.0.value.len() + 1
    }
    fn write_here<W: Write>(&self, mut writer: W) -> Result<()> {
        write_many(self.0.value.bytes(), &mut writer)?;
        (0 as u8).write_here(writer)
    }
}
impl Writable for &NullTerminated {
    fn size(&self) -> usize { (*self).size() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        (*self).write_here(writer)
    }
}