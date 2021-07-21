use std::str;
use std::convert::TryFrom;
use std::fmt;
use std::io::{Read, Seek, Result, Error, ErrorKind};

use super::bin::{read_struct, Readable};


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
            Ok(s) => Ok(BZString {
                value: s.to_owned()
            }),
            Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{}", e))),
        }
    }
}
impl Readable for BZString {
    type ReadableArgs = ();
    fn read<R: Read + Seek>(mut reader: R, _: ()) -> Result<BZString> {
        let length: u8 = read_struct(&mut reader)?;
        let mut chars: Vec<u8> = vec![0u8; length as usize];
        reader.read_exact(&mut chars)?;
        BZString::try_from(chars)
    }
}

pub struct NullTerminated(BZString);
impl From<NullTerminated> for BZString {
    fn from(s: NullTerminated) -> BZString {
        s.0
    }
}
impl Readable for NullTerminated {
    type ReadableArgs = ();
    fn read<R: Read + Seek>(mut reader: R, _: ()) -> Result<Self> {
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
