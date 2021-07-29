use std::io::{Read, Write, Result, copy};
use std::str;
use std::fmt;
use enumflags2::{bitflags, BitFlags};
use libflate::zlib;

use super::version::Version;
use super::v10x::{V10X, ToArchiveBitFlags, Versioned, RawDirRecord};
pub use super::v10x::V10XHeader;
pub use super::bzstring::BZString;


#[bitflags]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ArchiveFlag {
    #[doc = "The game may not load a BSA without this bit set."]
    pub IncludeDirectoryNames = 0x1,
    #[doc = "The game may not load a BSA without this bit set."]
    pub IncludeFileNames = 0x2,
    #[doc = "This does not mean all files are compressed. It means they are"]
    #[doc = "compressed by default."]
    pub CompressedArchive = 0x4,
    pub RetainDirectoryNames = 0x8,
    #[doc = "Unknown, but observed being set in official BSA files containing"]
    #[doc = "sounds (but not voices). Possibly instructs the game to retain"]
    #[doc = "file names in memory."]
    pub RetainFileNames = 0x10,
    pub RetainFileNameOffsets = 0x20,
    #[doc = "Hash values and numbers after the header are encoded big-endian."]
    pub Xbox360Archive = 0x40,
    pub Ux80  = 0x80,
    pub Ux100 = 0x100,
    pub Ux200 = 0x200,
    pub Ux400 = 0x400,
}
impl ToArchiveBitFlags for ArchiveFlag {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self> {
        BitFlags::from_bits_truncate(bits)
    }

    fn is_compressed_by_default() -> Self { ArchiveFlag::CompressedArchive }
    fn includes_file_names() -> Self { ArchiveFlag::IncludeFileNames }
    fn includes_dir_names() -> Self { ArchiveFlag::IncludeDirectoryNames }
}

pub type Header = V10XHeader<ArchiveFlag>;
pub enum V103T {}
pub type V103 = V10X<V103T, ArchiveFlag, RawDirRecord>;
impl Versioned for V103T {
    fn version() -> Version { Version::V103 }
    fn fmt_version(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BSA v103 file, format used by: TES IV: Oblivion")
    }

    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = zlib::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }
}
