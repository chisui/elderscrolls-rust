use std::fmt;
use std::str;

pub mod hash;
pub mod v103;
pub mod v104;
pub mod v105;


#[repr(C)]
#[derive(Debug)]
pub struct PreHeader {
    pub magic_number: MagicNumber,
    pub version: u32,
    pub offset: u32,
}

pub struct MagicNumber([u8; 4]);
impl fmt::Debug for MagicNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match str::from_utf8(&self.0) {
            Ok(v) => write!(f, "{}", v),
            Err(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug)]
pub struct Hash(u64);
