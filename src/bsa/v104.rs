use std::io::{Read, Write, Result};
use std::str;
use std::fmt;
use enumflags2::{bitflags, BitFlags};

use super::version::Version;
use super::v10x::{V10X, V10XHeader, RawDirRecord, ToArchiveBitFlags, Versioned};
use super::v103;
pub use super::v103::BZString;


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArchiveFlag {
    #[doc = "The game may not load a BSA without this bit set."]
    pub IncludeDirectoryNames = 0x1,
    #[doc = "The game may not load a BSA without this bit set."]
    pub IncludeFileNames = 0x2,
    #[doc = "This does not mean all files are compressed. It means they are"]
    #[doc = "compressed by default."]
    pub CompressedArchive = 0x4,
    pub RetainDirectoryNames = 0x8,
    pub RetainFileNames = 0x10,
    pub RetainFileNameOffsets = 0x20,
    #[doc = "Hash values and numbers after the header are encoded big-endian."]
    pub Xbox360Archive = 0x40,
    pub RetainStringsDuringStartup = 0x80,
    #[doc = "Embed File Names. Indicates the file data blocks begin with a"]
    #[doc = "bstring containing the full path of the file. For example, in"]
    #[doc = "\"Skyrim - Textures.bsa\" the first data block is"]
    #[doc = "$2B textures/effects/fxfluidstreamdripatlus.dds"]
    #[doc = "($2B indicating the name is 43 bytes). The data block begins"]
    #[doc = "immediately after the bstring."]
    pub EmbedFileNames = 0x100,
    #[doc = "This can only be used with COMPRESSED_ARCHIVE."]
    #[doc = "This is an Xbox 360 only compression algorithm."]
    pub XMemCodec = 0x200,
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
pub enum V104T{}
impl Versioned for V104T {
    fn version() -> Version { Version::V104 }
    fn fmt_version(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BSA v104 file, format used by: TES V: Skyrim, Fallout 3 and Fallout: New Vegas")
    }

    fn uncompress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64> {
        v103::V103T::uncompress(reader, writer)
    }
}

pub type V104 = V10X<V104T, ArchiveFlag, RawDirRecord>;
