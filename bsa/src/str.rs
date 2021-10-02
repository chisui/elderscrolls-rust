use std::{
    io::{Read, Write, Seek, SeekFrom, Result, Error, ErrorKind},
    str::{self, FromStr},
    convert::TryFrom,
};
use thiserror::Error;
use macro_attr_2018::macro_attr;
use newtype_derive_2018::*;

use super::bin::{read_struct, Readable, Writable, write_many};

#[derive(Debug, Error)]
#[error("BSA Strings may only be {0} chars or less long since their length is stored in a byte")]
pub struct StringToLong(usize);

macro_attr! {
    #[derive(Clone, Debug, PartialEq, Eq, NewtypeDeref!, NewtypeDerefMut!)]
    pub struct BString(String);
}
impl BString {
    pub fn new<B: AsRef<[u8]>> (chars: B) -> Result<Self> {
        from_utf8_io(chars)
    }
}
impl TryFrom<Vec<u8>> for BString {
    type Error = Error;
    fn try_from(chars: Vec<u8>) -> Result<Self> {
        from_utf8_io(chars)
    }
}
impl FromStr for BString {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
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
    type ReadableArgs = ();
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<Self> {
        let length: u8 = read_struct(&mut reader)?;
        let mut chars: Vec<u8> = vec![0u8; length as usize];
        reader.read_exact(&mut chars)?;
        Self::try_from(chars)
    }
}
impl Writable for BString {
    fn size(&self) -> usize {
        self.0.len() + 2 // length byte + chars + null
    }
    fn write_here<W: Write>(&self, mut out: W) -> Result<()> {
        (self.0.len() as u8).write_here(&mut out)?;
        write_many(self.0.bytes(), &mut out)?;
        (0 as u8).write_here(&mut out)
    }
}

macro_attr! {
    #[derive(Clone, Debug, PartialEq, Eq, NewtypeDeref!, NewtypeDerefMut!)]
    pub struct ZString(String);
}

impl ZString {
    pub fn new<B: AsRef<[u8]>> (chars: B) -> Result<Self> {
        from_utf8_io(chars)
    }
}
impl FromStr for ZString {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.to_owned()))
    }
}
impl TryFrom<Vec<u8>> for ZString {
    type Error = Error;
    fn try_from(chars: Vec<u8>) -> Result<Self> {
        from_utf8_io(chars)
    }
}

impl ToString for ZString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl Readable for ZString {
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<Self> {
        let mut chars: Vec<u8> = Vec::with_capacity(32);
        loop {
            let c: u8 = read_struct(&mut reader)?;
            if c == 0 {
                break;
            }
            chars.push(c);
        }
        Self::try_from(chars)
    }
}
impl Writable for ZString {
    fn size(&self) -> usize {
        self.0.len() + 1
    }
    fn write_here<W: Write>(&self, mut writer: W) -> Result<()> {
        write_many(self.0.bytes(), &mut writer)?;
        (0 as u8).write_here(writer)
    }
}


macro_attr! {
    #[derive(Clone, Debug, PartialEq, Eq, NewtypeDeref!, NewtypeDerefMut!)]
    pub struct BZString(String);
}

impl BZString {
    pub fn new<B: AsRef<[u8]>> (chars: B) -> Result<Self> {
        from_utf8_io(chars)
    }
}
impl TryFrom<Vec<u8>> for BZString {
    type Error = Error;
    fn try_from(chars: Vec<u8>) -> Result<Self> {
        from_utf8_io(chars)
    }
}
impl FromStr for BZString {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        check_len(s, 254)?;
        Ok(Self(s.to_owned()))
    }
}

impl ToString for BZString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl Readable for BZString {
    type ReadableArgs = ();
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<Self> {
        let length: u8 = read_struct(&mut reader)?;
        let mut chars: Vec<u8> = vec![0u8; (length - 1) as usize]; // length field includes null.
        reader.read_exact(&mut chars)?;
        reader.seek(SeekFrom::Current(1))?; // skip null byte.
        Self::try_from(chars)
    }
}
impl Writable for BZString {
    fn size(&self) -> usize {
        self.0.len() + 2 // length byte + chars + null
    }
    fn write_here<W: Write>(&self, mut out: W) -> Result<()> {
        (self.0.len() as u8 + 1).write_here(&mut out)?;
        write_many(self.0.bytes(), &mut out)?;
        (0 as u8).write_here(&mut out)
    }
}

fn from_utf8_io<B: AsRef<[u8]>, S: FromStr<Err = Error>>(chars: B) -> Result<S> {
    match str::from_utf8(chars.as_ref()) {
        Ok(s) => S::from_str(s),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
    }   
}

fn check_len(s: &str, max_len: usize) -> Result<()> {
    if s.len() > max_len {
        Err(Error::new(ErrorKind::InvalidData, StringToLong(max_len)))
    } else {
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use super::*;
 
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
        len_check::<BString>();
    }

    #[test]
    fn bzstring_len_check() {
        len_check::<BZString>();
    }

    fn len_check<S: FromStr<Err = Error> + Debug>() {
        let s: String = (0..500).map(|_| 'a').collect();
        match S::from_str(&s) {
            Ok(s)  => panic!("expected error but got error but got {:?}", s),
            Err(_) => {},
        };
    }

    fn write_read_identity<A: Writable + Readable<ReadableArgs = ()> + Debug + Eq>(expected: A) {
        let actual = write_read(&expected);

        assert_eq!(expected, actual)
    }

    fn write_read<A: Writable + Readable<ReadableArgs = ()> + Debug>(val: &A) -> A {
        use std::io::Cursor;
        let mut out = Cursor::new(Vec::<u8>::new());
        val.write_here(&mut out)
            .unwrap_or_else(|err| panic!("could not write {:?}: {}", val, err));
        let mut input = Cursor::new(out.into_inner());
        A::read_here0(&mut input)
            .unwrap_or_else(|err| panic!("could not read {:?}: {}", val, err))
    }
}