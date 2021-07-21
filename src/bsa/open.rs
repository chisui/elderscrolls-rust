use std::io::{Result, Read, Seek, Error, ErrorKind};

use super::archive;
use super::bin::Readable;
use super::version::Version;
use super::v105;


pub enum WrappedBsa<R: Read + Seek> {
    V105(v105::Bsa<R>),
}
impl<R: Read + Seek> WrappedBsa<R> {
    pub fn open(mut reader: R) -> Result<WrappedBsa<R>> {
        let version = Version::read(&mut reader, ())?;
        match version {
            Version::V105 => v105::Bsa::<R>::open(reader).map(WrappedBsa::V105),
            v => Err(Error::new(ErrorKind::InvalidData, format!("unsupported version: {}", v))),
        }
    }
}
impl<R: Read + Seek> archive::Bsa for WrappedBsa<R> {
    fn version(&self) -> Result<Version> {
        match self {
            WrappedBsa::V105(_) => Ok(Version::V105),
        }
    }
}
