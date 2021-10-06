use std::{
    io::{self, Read, Write},
    str::{self, FromStr},
    convert::TryFrom,
};
use thiserror::Error;

use crate::bin::{Readable, VarSize, Writable, read_struct};


#[derive(Debug, Error)]
pub enum StrError {
    #[error("string may only be {0} chars or less long since their length is stored in a byte")]
    TooLong(usize),
    #[error("{0}")]
    Utf8Error(#[from] str::Utf8Error),
}
impl From<StrError> for io::Error {
    fn from(err: StrError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, err)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BString(String);

impl TryFrom<Vec<u8>> for BString {
    type Error = StrError;
    fn try_from(chars: Vec<u8>) -> Result<Self, StrError> {
        from_utf8(chars)
    }
}
impl FromStr for BString {
    type Err = StrError;
    fn from_str(s: &str) -> Result<Self, StrError> {
        check_len(s, 255)?;
        Ok(Self(s.to_owned()))
    }
}
impl ToString for BString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl Readable for BString {
    fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let length: u8 = read_struct(&mut reader)?;
        let mut chars: Vec<u8> = vec![0u8; length as usize];
        reader.read_exact(&mut chars)?;
        let s = Self::try_from(chars)?;
        Ok(s)
    }
}
impl VarSize for BString {
    fn size(&self) -> usize {
        self.0.len() + 2 // length byte + chars + null
    }
}
impl Writable for BString {
    fn write<W: Write>(&self, mut out: W) -> io::Result<()> {
        (self.0.len() as u8).write(&mut out)?;
        self.0.as_bytes().write(&mut out)?;
        (0 as u8).write(&mut out)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZString(String);
impl ZString {
    pub fn new<B: AsRef<[u8]>> (chars: B) -> Result<Self, StrError> {
        from_utf8(chars)
    }
}
impl FromStr for ZString {
    type Err = StrError;
    fn from_str(s: &str) -> Result<Self, StrError> {
        Ok(Self(s.to_owned()))
    }
}
impl TryFrom<Vec<u8>> for ZString {
    type Error = StrError;
    fn try_from(chars: Vec<u8>) -> Result<Self, StrError> {
        from_utf8(chars)
    }
}

impl ToString for ZString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl Readable for ZString {
    fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut chars: Vec<u8> = Vec::with_capacity(32);
        loop {
            let c: u8 = read_struct(&mut reader)?;
            if c == 0 {
                break;
            }
            chars.push(c);
        }
        let s = Self::try_from(chars)?;
        Ok(s)
    }
}
impl VarSize for ZString {
    fn size(&self) -> usize {
        self.0.len() + 1
    }
}
impl Writable for ZString {
    fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        self.0.as_bytes().write(&mut writer)?;
        (0 as u8).write(writer)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BZString(String);

impl BZString {
    pub fn new<B: AsRef<[u8]>> (chars: B) -> Result<Self, StrError> {
        from_utf8(chars)
    }
}
impl TryFrom<Vec<u8>> for BZString {
    type Error = StrError;
    fn try_from(chars: Vec<u8>) -> Result<Self, StrError> {
        from_utf8(chars)
    }
}
impl FromStr for BZString {
    type Err = StrError;
    fn from_str(s: &str) -> Result<Self, StrError> {
        check_len(s, 254)?;
        Ok(Self(s.to_owned()))
    }
}

impl ToString for BZString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl VarSize for BZString {
    fn size(&self) -> usize {
        self.0.len() + 2 // length byte + chars + null
    }
}
impl Readable for BZString {
    fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let len = u8::read(&mut reader)?;
        let mut chars: Vec<u8> = vec![0u8; (len - 1) as usize]; // length field includes null.
        reader.read_exact(&mut chars)?;
        u8::read(&mut reader)?; // skip null byte
        let s = Self::try_from(chars)?;
        Ok(s)
    }
}
impl Writable for BZString {
    fn write<W: Write>(&self, mut out: W) -> io::Result<()> {
        (self.0.len() as u8 + 1).write(&mut out)?;
        self.0.as_bytes().write(&mut out)?;
        (0 as u8).write(&mut out)
    }
}

fn from_utf8<B: AsRef<[u8]>, S: FromStr<Err = StrError>>(chars: B) -> Result<S, StrError> {
    let s = str::from_utf8(chars.as_ref())?;
    S::from_str(s)
}

fn check_len(s: &str, max_len: usize) -> Result<(), StrError> {
    if s.len() > max_len {
        Err(StrError::TooLong(max_len))
    } else {
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use super::*;
    use crate::bin::test::*;
 
    #[test]
    fn write_read_identity_bstring_zero_len() {
        write_read_identity(BString("".to_owned()));
    }

    #[test]
    fn write_read_identity_zstring_zero_len() {
        write_read_identity(ZString("".to_owned()));
    }

    #[test]
    fn write_read_identity_bzstring_zero_len() {
        write_read_identity(BZString("".to_owned()));
    }

    #[test]
    fn write_read_identity_bstring_some_chars() {
        write_read_identity(BString("asdf_basdf".to_owned()));
    }

    #[test]
    fn write_read_identity_zstring_some_chars() {
        write_read_identity(ZString("asdf_basdf".to_owned()));
    }

    #[test]
    fn write_read_identity_bzstring_some_chars() {
        write_read_identity(BZString("asdf_basdf".to_owned()));
    }

    #[test]
    fn bstring_len_check() {
        len_check::<BString>(255);
    }

    #[test]
    fn bzstring_len_check() {
        len_check::<BZString>(254);
    }

    fn len_check<S: FromStr<Err = StrError> + Debug>(max_len: usize) {
        let s: String = (0..500).map(|_| 'a').collect();
        match S::from_str(&s) {
            Err(StrError::TooLong(l)) => assert_eq!(max_len, l, "max_len"),
            res  => panic!("expected String too long error but got error but got {:?}", res),
        };
    }
}