use std::io::{Read, Seek, SeekFrom, Result, Write, copy};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::size_of;
use std::str;
use std::fmt;
use bytemuck::{Pod, Zeroable};
use enumflags2::{bitflags, BitFlags, BitFlag};

use super::bin::{self, read_struct, Readable};
use super::hash::{Hash, hash_v10x};
use super::version::Version;
use super::magicnumber::MagicNumber;
use super::archive::{Bsa, BsaDir, BsaFile, FileId};
pub use super::bzstring::{BZString, NullTerminated};


pub trait ToArchiveBitFlags: BitFlag + fmt::Debug {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self>;
    fn from_archive_bit_flags(flags: BitFlags<Self>) -> u32;

    fn is_compressed_by_default() -> Self;
    
    fn includes_file_names() -> Self;
    
    fn includes_dir_names() -> Self;
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
impl<AF: ToArchiveBitFlags> Default for V10XHeader<AF> {
    fn default() -> Self {
        let mut h: Self = RawHeader::zeroed().into();
        h.offset = (size_of::<MagicNumber>() + size_of::<u32>() + size_of::<RawHeader>()) as u32;
        h
    }
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
impl<AF: ToArchiveBitFlags> From<V10XHeader<AF>> for RawHeader {
    fn from(h: V10XHeader<AF>) -> Self {
        let V10XHeader {
            offset,
            archive_flags,
            folder_count,
            file_count,
            total_folder_name_length,
            total_file_name_length,
            file_flags,
            padding,
        } = h;
        Self {
            offset,
            archive_flags: ToArchiveBitFlags::from_archive_bit_flags(archive_flags),
            folder_count,
            file_count,
            total_folder_name_length,
            total_file_name_length,
            file_flags: file_flags.bits(),
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
    fn offset(_: &()) -> Option<usize> {
        Some(size_of::<MagicNumber>() + size_of::<Version>())
    }
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<V10XHeader<AF>> {
        bin::read_struct::<RawHeader, _>(&mut reader)
            .map(V10XHeader::<AF>::from)
    }
}
impl<AF: ToArchiveBitFlags> bin::Writable for V10XHeader<AF> {
    fn size(&self) -> usize { size_of::<RawHeader>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        let raw: RawHeader = (*self).into();
        bin::write_struct(&raw, writer)
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


pub struct V10XArchive<R, T, AF: ToArchiveBitFlags, RDR> {
    pub reader: R,
    pub header: V10XHeader<AF>,
    phantom_t: PhantomData<T>,
    phantom_af: PhantomData<AF>,
    phantom_rdr: PhantomData<RDR>,
}
impl<R: Read + Seek, T, AF: ToArchiveBitFlags, RDR> V10XArchive<R, T, AF, RDR> {
    pub fn open(mut reader: R) -> Result<Self> {
        let header = V10XHeader::<AF>::read0(&mut reader)?;
        Ok(V10XArchive {
            reader,
            header,
            phantom_t: PhantomData,
            phantom_af: PhantomData,
            phantom_rdr: PhantomData,
        })
    }

    fn offset_file_names(&self) -> usize {
            
        let folder_records_size = size_of::<RDR>() * self.header.folder_count as usize;
        let folder_names_size = if self.header.has(AF::includes_dir_names()) {
            self.header.total_folder_name_length as usize
            + self.header.folder_count as usize // total_folder_name_length does not include size byte
        } else {
            0
        };
        self.offset_after_header() + folder_records_size + folder_names_size + self.header.file_count as usize * size_of::<FileRecord>()
    }

    fn offset_after_header(&self) -> usize {
        size_of::<MagicNumber>() + size_of::<Version>() + size_of::<RawHeader>()
    }

    fn read_file_names(&mut self) -> Result<HashMap<Hash, BZString>> {
        self.reader.seek(SeekFrom::Start(self.offset_file_names() as u64))?;
        Ok(if self.header.has(AF::includes_file_names()) {
            let names = NullTerminated::read_many0(&mut self.reader, self.header.file_count as usize)?;
            names.iter()
                .map(BZString::from)
                .map(|name| (hash_v10x(name.value.as_str()), name.clone()))
                .collect()
        } else {
            HashMap::new()
        })
    }
}
pub trait Versioned {
    fn fmt_version(f: &mut fmt::Formatter<'_>) -> fmt::Result;

    fn version() -> Version;

    fn uncompress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64>;
}
impl<R, T: Versioned, AF: ToArchiveBitFlags + fmt::Debug, RDR: Readable<ReadableArgs=()> + Sized> fmt::Display for V10XArchive<R, T, AF, RDR> {
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt_version(&mut f)?;
        self.header.fmt(f)
    }
}
impl<R: Read + Seek, T: Versioned, AF: ToArchiveBitFlags + fmt::Debug, RDR: Readable<ReadableArgs=()> + Sized + Copy> Bsa for V10XArchive<R, T, AF, RDR>
where DirRecord: From<RDR> {
    type Header = V10XHeader<AF>;

    fn version(&self) -> Version {
        T::version()
    }

    fn header(&self) -> Self::Header {
        self.header
    }

    fn read_dirs(&mut self) -> Result<Vec<BsaDir>> {
        let hasdir_name = self.header.has(AF::includes_file_names());
        self.reader.seek(SeekFrom::Start(self.offset_after_header() as u64))?;
        let raw_dirs = RDR::read_many0(&mut self.reader, self.header.folder_count as usize)?;
        let file_names = self.read_file_names()?;

        raw_dirs.iter()
            .map(|dir| DirRecord::from(*dir) )
            .map(|dir| {
                self.reader.seek(SeekFrom::Start(dir.offset as u64 - self.header.total_file_name_length as u64))?;
                let dir_content = FolderContentRecord::read(&mut self.reader, &(hasdir_name, dir.file_count))?;
                Ok(BsaDir {

                    name: dir_content.name
                        .map(FileId::StringId)
                        .unwrap_or(FileId::HashId(dir.name_hash)),
                    
                    files: dir_content.file_records.iter().map(|file| {
                        
                        let compressed = if self.header.has(AF::is_compressed_by_default()) {
                            !file.is_compression_bit_set()
                        } else {
                            file.is_compression_bit_set()
                        };

                        BsaFile {
                            name: file_names.get(&file.name_hash)
                                .map(BZString::clone)
                                .map(FileId::StringId)
                                .unwrap_or(FileId::HashId(file.name_hash)),
                            compressed,
                            offset: file.offset as u64,
                            size: file.size,
                        }

                    }).collect::<Vec<_>>(),
                })
            })
            .collect::<Result<Vec<_>>>()
    }


    fn extract<W: Write>(&mut self, file: BsaFile, mut writer: W) -> Result<()> {
        self.reader.seek(SeekFrom::Start(file.offset))?;

        // skip name field
        if self.header.has(AF::includes_file_names()) {
            let name_len: u8 = read_struct(&mut self.reader)?;
            self.reader.seek(SeekFrom::Current(name_len as i64))?;
        }
    
        if file.compressed {
            // skip uncompressed size field
            self.reader.seek(SeekFrom::Current(4))?;
    
            let sub_reader = (&mut self.reader).take(file.size as u64);
            T::uncompress(sub_reader, writer)?;
        } else {
            let mut sub_reader = (&mut self.reader).take(file.size as u64);
            copy(&mut sub_reader, &mut writer)?;
        }
        Ok(())
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct DirRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub offset: u32,
}
impl Readable for DirRecord {
    fn read_here<R: Read>(reader: R, _: &()) -> Result<DirRecord> {
        read_struct(reader)
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FileRecord {
    pub name_hash: Hash,
    pub size: u32,
    pub offset: u32,
}
impl FileRecord {
    pub fn is_compression_bit_set(&self) -> bool {
        (self.size & 0x40000000) == 0x40000000
    }
}
impl bin::Readable for FileRecord {
    fn read_here<R: Read + Seek>(mut reader: R, _: &()) -> Result<FileRecord> {
        bin::read_struct(&mut reader)
    }
}


#[derive(Debug)]
pub struct FolderContentRecord {
    pub name: Option<BZString>,
    pub file_records: Vec<FileRecord>,
}
impl Readable for FolderContentRecord {
    type ReadableArgs = (bool, u32);
    fn read_here<R: Read + Seek>(mut reader: R, (has_name, file_count): &(bool, u32)) -> Result<FolderContentRecord> {
        let name = if *has_name {
            let n = BZString::read(&mut reader, &())?;
            Some(n)
        } else {
            None
        };
        let file_records = FileRecord::read_many0(reader, *file_count as usize)?;
        Ok(FolderContentRecord {
            name,
            file_records,
        })
    }
}
