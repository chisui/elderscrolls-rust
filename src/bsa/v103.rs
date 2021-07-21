use std::io::{Read, Result};
use std::str;
use bytemuck::{Pod, Zeroable};
use enumflags2::{bitflags, BitFlags, BitFlag};

use super::bin;
pub use super::hash::Hash;
pub use super::bzstring::BZString;


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

pub trait ToArchiveBitFlags: BitFlag {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self>;
}
impl ToArchiveBitFlags for ArchiveFlags {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self> {
        BitFlags::from_bits_truncate(bits)
    }
}

#[bitflags]
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FileFlag {
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
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct RawHeader {
    pub _offset: u32,
    pub archive_flags: u32,
    pub folder_count: u32,
    pub file_count: u32,
    pub total_folder_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: u16,
    pub padding: u16,
}

#[derive(Debug)]
pub struct V10XHeader<AF: BitFlag> {
    pub _offset: u32,
    pub archive_flags: BitFlags<AF>,
    pub folder_count: u32,
    pub file_count: u32,
    pub total_folder_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: BitFlags<FileFlag>,
    pub padding: u16,
}
impl<AF: ToArchiveBitFlags> From<RawHeader> for V10XHeader<AF> {
    fn from(raw: RawHeader) -> V10XHeader<AF> {
        let RawHeader {
            _offset,
            archive_flags,
            folder_count,
            file_count,
            total_folder_name_length,
            total_file_name_length,
            file_flags,
            padding,
        } = raw;
        Self {
            _offset,
            archive_flags: ToArchiveBitFlags::to_archive_bit_flags(archive_flags),
            folder_count,
            file_count,
            total_folder_name_length,
            total_file_name_length,
            file_flags: BitFlags::from_bits_truncate(file_flags),
            padding,
        }   
    }
}
impl<AF: ToArchiveBitFlags> V10XHeader<AF> {
    pub fn has_archive_flag(&self, f: AF) -> bool {
        self.archive_flags.contains(f)
    }
    pub fn has_file_flag(&self, f: FileFlag) -> bool {
        self.file_flags.contains(f)
    }
}
impl<AF: ToArchiveBitFlags> bin::Readable for V10XHeader<AF> {
    type ReadableArgs = ();
    fn read<R: Read>(mut reader: R, _: &()) -> Result<V10XHeader<AF>> {
        let raw: RawHeader = bin::read_struct(&mut reader)?;
        Ok(V10XHeader::<AF>::from(raw))
    }
}

#[allow(dead_code)]
pub type Header = V10XHeader<ArchiveFlags>;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FolderRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub offset: u32,
}
