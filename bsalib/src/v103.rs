use std::{
    io::{BufReader, Read, Seek, Write, Result, copy},
    str,
    path::Path,
    fs::File,
};
use enumflags2::{bitflags, BitFlags};
use libflate::zlib;

use crate::{
    version::Version10X,
    v10x::{
        V10XReader,
        V10XHeader,
        V10XWriter,
        V10XWriterOptions,
        ToArchiveBitFlags,
        Versioned,
        DirRecord
    },
};


#[bitflags]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ArchiveFlag {
    #[doc = "The game may not load a BSA without this bit set."]
    IncludeDirectoryNames = 0x1,
    #[doc = "The game may not load a BSA without this bit set."]
    IncludeFileNames = 0x2,
    #[doc = "This does not mean all files are compressed. It means they are"]
    #[doc = "compressed by default."]
    CompressedArchive = 0x4,
    RetainDirectoryNames = 0x8,
    #[doc = "Unknown, but observed being set in official BSA files containing"]
    #[doc = "sounds (but not voices). Possibly instructs the game to retain"]
    #[doc = "file names in memory."]
    RetainFileNames = 0x10,
    RetainFileNameOffsets = 0x20,
    #[doc = "Hash values and numbers after the header are encoded big-endian."]
    Xbox360Archive = 0x40,
    Ux80  = 0x80,
    Ux100 = 0x100,
    Ux200 = 0x200,
    Ux400 = 0x400,
}
impl ToArchiveBitFlags for ArchiveFlag {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self> {
        BitFlags::from_bits_truncate(bits)
    }
    fn from_archive_bit_flags(flags: BitFlags<Self>) -> u32 { 
        flags.bits()
    }
    

    fn is_compressed_by_default() -> Self { ArchiveFlag::CompressedArchive }
    fn includes_file_names() -> Self { ArchiveFlag::IncludeFileNames }
    fn includes_dir_names() -> Self { ArchiveFlag::IncludeDirectoryNames }
}

pub enum V103 {}
impl Versioned for V103 {
    fn version() -> Version10X { Version10X::V103 }
   
    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = zlib::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }

    fn compress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut encoder = zlib::Encoder::new(&mut writer)?;
        copy(&mut reader, &mut encoder)
    }
}

pub fn open<P>(path: P) -> Result<BsaReader<BufReader<File>>>
where P: AsRef<Path> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    read(buf)
}
pub fn read<R>(reader: R) -> Result<BsaReader<R>>
where R: Read + Seek {
    BsaReader::read(reader)
}

pub type Header = V10XHeader<ArchiveFlag>;
pub type BsaReader<R> = V10XReader<R, V103, ArchiveFlag, DirRecord>;
pub type BsaWriter = V10XWriter<V103, ArchiveFlag, DirRecord>;
pub type BsaWriterOptions = V10XWriterOptions<ArchiveFlag>;
