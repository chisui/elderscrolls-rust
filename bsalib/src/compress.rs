use std::io::{Read, Write, Result, copy};

use libflate::zlib;


pub trait Compression {
    fn uncompress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64>;

    fn compress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64>;
}

pub enum ZLib {}
impl Compression for ZLib {
    
    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = zlib::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }

    fn compress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut encoder = zlib::Encoder::new(&mut writer)?;
        let size = copy(&mut reader, &mut encoder)?;
        encoder.finish().into_result()?;
        Ok(size)
    }
}

pub enum Lz4 {}
impl Compression for Lz4 {
    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = lz4::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }

    fn compress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut encoder = lz4::EncoderBuilder::new()
            .auto_flush(true)
            .build(&mut writer)?;
        copy(&mut reader, &mut encoder)
    }
}