use std::io::{Read, Seek, SeekFrom, Result, Write, copy};
use std::mem::size_of;
use std::str;
use std::fmt;
use bytemuck::{Pod, Zeroable};
use enumflags2::{bitflags, BitFlags, BitFlag};

use super::bin;
use super::version::{Version, MagicNumber};
use super::archive::{BsaFile};
pub use super::bzstring::BZString;


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
impl ToArchiveBitFlags for ArchiveFlag {
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
    pub offset: u32,
    pub archive_flags: u32,
    pub folder_count: u32,
    pub file_count: u32,
    pub total_folder_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: u16,
    pub padding: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct V10XHeader<AF: BitFlag> {
    pub offset: u32,
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
            offset,
            archive_flags,
            folder_count,
            file_count,
            total_folder_name_length,
            total_file_name_length,
            file_flags,
            padding,
        } = raw;
        Self {
            offset,
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
pub trait Has<T> {
    fn has(&self, t: T) -> bool;
}
impl<AF: ToArchiveBitFlags> Has<AF> for V10XHeader<AF> {
    fn has(&self, f: AF) -> bool {
        self.archive_flags.contains(f)
    }
}
impl<AF: ToArchiveBitFlags> Has<FileFlag> for V10XHeader<AF> {
    fn has(&self, f: FileFlag) -> bool {
        self.file_flags.contains(f)
    }
}
impl<AF: ToArchiveBitFlags + fmt::Debug> bin::Readable for V10XHeader<AF> {
    fn offset(_: &()) -> Option<u64> {
        Some(size_of::<MagicNumber>() as u64 + size_of::<Version>() as u64)
    }
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<V10XHeader<AF>> {
        let raw: RawHeader = bin::read_struct(&mut reader)?;
        Ok(V10XHeader::<AF>::from(raw))
    }
}
impl<AF: ToArchiveBitFlags + fmt::Debug> fmt::Display for V10XHeader<AF> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Folders: {}", self.folder_count)?;
        writeln!(f, "Files:   {}", self.file_count)?;
        writeln!(f, "Archive flags:")?;
        for flag in self.archive_flags.iter() {
            writeln!(f, "    {:?}", flag)?;
        }
        writeln!(f, "File flags:")?;
        for flag in self.file_flags.iter() {
            writeln!(f, "    {:?}", flag)?;
        }
        Ok(())
    }
}

pub type Header = V10XHeader<ArchiveFlag>;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FolderRecord {
    pub name_hash: u64,
    pub file_count: u32,
    pub offset: u32,
}

pub fn extract<R: Read + Seek, W: Write>(includes_name: bool, file: BsaFile, mut reader: R, mut writer: W) -> Result<()> {
    reader.seek(SeekFrom::Start(file.offset))?;

    // skip name field
    if includes_name {
        let name_len: u8 = bin::read_struct(&mut reader)?;
        reader.seek(SeekFrom::Current(name_len as i64))?;
    }
    
    if file.compressed {
        // skip uncompressed size field
        reader.seek(SeekFrom::Current(4))?;
        
        let mut sub_reader = reader.take(file.size as u64);
        let mut decoder = libflate::zlib::Decoder::new(&mut sub_reader)?;
        copy(&mut decoder, &mut writer)?;
    } else {
        let mut sub_reader = reader.take(file.size as u64);
        copy(&mut sub_reader, &mut writer)?;
    }
    Ok(())
}

