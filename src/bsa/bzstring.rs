use std::str;
use std::convert::TryFrom;
use std::fmt;
use std::io::{Read, Seek, Result, Error, ErrorKind};

use super::bin::{read_struct, Readable};


#[derive(Clone)]
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
            Err(e) => Err(Error::new(ErrorKind::InvalidData, format!("{}, {:x?}", e, chars))),
        }
    }
}
impl Readable for BZString {
    type ReadableArgs = ();
    fn read_here<R: Read + Seek>(mut reader: R, _: ()) -> Result<BZString> {
        let length: u8 = read_struct(&mut reader)?;
        let mut chars: Vec<u8> = vec![0u8; length as usize];
        reader.read_exact(&mut chars)?;
        match BZString::try_from(chars) {
            Ok(s) => Ok(s),
            Err(e) => {
                let pos = reader.stream_position()?;
                Err(Error::new(ErrorKind::InvalidData, format!("{} at: {:08}", e, pos)))
            },
        }
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
impl Readable for NullTerminated {
    fn read_here<R: Read + Seek>(mut reader: R, _: ()) -> Result<Self> {
        let mut chars: Vec<u8> = Vec::with_capacity(32);
        loop {
            let c: u8 = read_struct(&mut reader)?;
            if c == 0 {
                break;
            }
            chars.push(c);
        }
        match BZString::try_from(chars) {
            Ok(s) => Ok(NullTerminated(s)),
            Err(e) => {
                let pos = reader.stream_position()?;
                Err(Error::new(ErrorKind::InvalidData, format!("{} at: {:08x}", e, pos)))
            },
        }
    }
}
