use std::io::{Read, Write, Result, copy};

use libflate::zlib;


/// The Compression trait provides a way to pass around a type reference to a specific
/// compression algorithm.
///
/// [`uncompress()`] has to be the inverse of [`compress`] and vice verca.
pub trait Compression {

    /// Read everything from a reader and write the compressed data to a writer.
    /// The result is the number of bytes written to the writer.
    fn compress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64>;
    
    /// Uncompress the data from the reader and write it to the writer.
    /// the result is the number of bytes written to the writer.
    fn uncompress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64>;
}

/// The zlib compression algorithm as implemented by [`libflate::zlib`].
pub enum ZLib {}
impl Compression for ZLib {

    fn compress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut encoder = zlib::Encoder::new(&mut writer)?;
        let size = copy(&mut reader, &mut encoder)?;
        encoder.finish().into_result()?;
        Ok(size)
    }

    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = zlib::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }    
}

/// The lz4 compression algorithm as implemented by [`lz4`].
pub enum Lz4 {}
impl Compression for Lz4 {
    fn compress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut encoder = lz4::EncoderBuilder::new()
            .auto_flush(true)
            .build(&mut writer)?;
        copy(&mut reader, &mut encoder)
    }

    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = lz4::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }
}