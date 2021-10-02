use std::{
    io::{Read, Seek, Write, Result},
    str,
    fmt,
};
use enumflags2::{bitflags, BitFlags};

use super::{
    version::{Version, Version10X},
    v10x::{
        V10XReader,
        V10XHeader,
        V10XWriter,
        V10XWriterOptions,
        DirRecord,
        ToArchiveBitFlags,
        Versioned,
    },
    v103,
};


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArchiveFlag {
    #[doc = "The game may not load a BSA without this bit set."]
    IncludeDirectoryNames = 0x1,
    #[doc = "The game may not load a BSA without this bit set."]
    IncludeFileNames = 0x2,
    #[doc = "This does not mean all files are compressed. It means they are"]
    #[doc = "compressed by default."]
    CompressedArchive = 0x4,
    RetainDirectoryNames = 0x8,
    RetainFileNames = 0x10,
    RetainFileNameOffsets = 0x20,
    #[doc = "Hash values and numbers after the header are encoded big-endian."]
    Xbox360Archive = 0x40,
    RetainStringsDuringStartup = 0x80,
    #[doc = "Embed File Names. Indicates the file data blocks begin with a"]
    #[doc = "bstring containing the full path of the file. For example, in"]
    #[doc = "\"Skyrim - Textures.bsa\" the first data block is"]
    #[doc = "$2B textures/effects/fxfluidstreamdripatlus.dds"]
    #[doc = "($2B indicating the name is 43 bytes). The data block begins"]
    #[doc = "immediately after the bstring."]
    EmbedFileNames = 0x100,
    #[doc = "This can only be used with COMPRESSED_ARCHIVE."]
    #[doc = "This is an Xbox 360 only compression algorithm."]
    XMemCodec = 0x200,
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
    fn embed_file_names() -> Option<Self> { Some(ArchiveFlag::EmbedFileNames) }
}

pub type Header = V10XHeader<ArchiveFlag>;
pub type BsaReader<R> = V10XReader<R, V104, ArchiveFlag, DirRecord>;
pub enum V104 {}
impl V104 {
    pub fn open<R>(reader: R) -> Result<BsaReader<R>>
    where R: Read + Seek {
        BsaReader::open(reader)
    }
}
impl Versioned for V104 {
    fn version() -> Version { Version::V10X(Version10X::V104) }
    fn fmt_version(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BSA v104 file, format used by: TES V: Skyrim, Fallout 3 and Fallout: New Vegas")
    }

    fn uncompress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64> {
        v103::V103::uncompress(reader, writer)
    }

    fn compress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64> {
        v103::V103::compress(reader, writer)
    }
}

pub type BsaWriter = V10XWriter<V104, ArchiveFlag, DirRecord>;
pub type BsaWriterOptions = V10XWriterOptions<ArchiveFlag>;
