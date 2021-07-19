use std::str;
use enumflags2::bitflags;

use super::v103::V2Header;
pub use super::{MagicNumber, Hash};
pub use super::v103::{FileFlags, FolderRecord};


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArchiveFlags {
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

pub type Header = V2Header<ArchiveFlags>;

#[repr(C)]
#[derive(Debug)]
pub struct FileRecord {
    pub name_hash: Hash,
    pub size: u32,
    pub offset: u32,
}
impl FileRecord {
    pub fn is_compressed(&self) -> bool {
        (self.size & 0x40000000) == 0x40000000
    }
}
