use std::str;
use enumflags2::{bitflags, BitFlags, BitFlag};

pub use super::{MagicNumber, Hash};


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

#[bitflags]
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FileFlags {
    pub Meshes = 0x1,
    pub Textures = 0x2,
    pub Menus = 0x4,
    pub Sounds = 0x8,
    pub Voices = 0x10,
    pub Shaders = 0x20,
    pub Trees = 0x40,
    pub Fonts = 0x80,
    pub Miscellaneous = 0x100,
}

#[repr(C)]
#[derive(Debug)]
pub struct V2Header<A: BitFlag> { 
    pub archive_flags: BitFlags<A>,
    pub folder_count: u32,
    pub file_count: u32,
    pub total_folder_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: BitFlags<FileFlags>,
    pub padding: u16,
}

pub type Header = V2Header<ArchiveFlags>;

#[repr(C)]
#[derive(Debug)]
pub struct FolderRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub offset: u32,
}
