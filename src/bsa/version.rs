use std::io::{Read, Result, Error, ErrorKind};
use std::fmt;

use super::bin;


#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
pub enum Version {
    V1, // TES3
    V103, // TES4
    V104, // F3, FNV, TES5
    V105, // TES5se
    V2, // F4 F76
}
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Version::V1   => "v001",
            Version::V103 => "v103",
            Version::V104 => "v104",
            Version::V105 => "v105",
            Version::V2   => "v2",
        })
    }
}
impl bin::Readable for Version {
    type ReadableArgs = ();
    fn read<R: Read>(mut buffer: R, _: ()) -> Result<Self> {
        let mut magic_number = [0; 4];
        buffer.read(&mut magic_number[..])?;
        if magic_number == [0,0,1,0] {
            Ok(Version::V1)
        } else { // just treat everything else as V10X Since Version field should match
            let mut version = [0; 4];
            buffer.read(&mut version[..])?;
            match version[0] {
                103 => Ok(Version::V103),
                104 => Ok(Version::V104),
                105 => Ok(Version::V105),
                _   => Err(Error::new(ErrorKind::InvalidData, format!("Unknown Version {}", version[0]))),
            }
        }
    }
}
