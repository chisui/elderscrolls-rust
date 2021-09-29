use std::io::{Read, Seek, SeekFrom, Result, Write, copy};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::size_of;
use std::str::{self, FromStr};
use std::fmt;
use bytemuck::{Pod, Zeroable};
use enumflags2::{bitflags, BitFlags, BitFlag};

use super::bin::{self, read_struct, Readable, Writable};
use super::hash::{Hash, hash_v10x};
use super::version::Version;
use super::magicnumber::MagicNumber;
use super::archive::{Bsa, BsaDir, BsaFile, FileId, BsaDirSource, BsaFileSource, BsaWriter};
pub use super::bzstring::{BZString, NullTerminated};


pub trait ToArchiveBitFlags: BitFlag + fmt::Debug {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self>;
    fn from_archive_bit_flags(flags: BitFlags<Self>) -> u32;

    fn is_compressed_by_default() -> Self;
    
    fn includes_file_names() -> Self;
    
    fn includes_dir_names() -> Self;

    fn embed_file_names() -> Option<Self> {
        None
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
    phantom_rdr: PhantomData<RDR>,
}
impl<R: Read + Seek, T, AF: ToArchiveBitFlags, RDR> V10XArchive<R, T, AF, RDR> {
    pub fn open(mut reader: R) -> Result<Self> {
        let header = V10XHeader::<AF>::read0(&mut reader)?;
        Ok(V10XArchive {
            reader,
            header,
            phantom_t: PhantomData,
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

    fn compress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64>;
}
impl<R, T, AF, RDR> fmt::Display for V10XArchive<R, T, AF, RDR>
where
    T: Versioned,
    AF: ToArchiveBitFlags + fmt::Debug,
{
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt_version(&mut f)?;
        self.header.fmt(f)
    }
}
impl<R, T, AF, RDR> Bsa for V10XArchive<R, T, AF, RDR>
where
    R: Read + Seek,
    T: Versioned,
    AF: ToArchiveBitFlags + fmt::Debug,
    RDR: Readable<ReadableArgs=()> + Sized + Copy,
    DirRecord: From<RDR>,
{
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
                self.reader.seek(SeekFrom::Start(
                    dir.offset as u64 - self.header.total_file_name_length as u64))?;
                let dir_content = FolderContentRecord::read(&mut self.reader, &(hasdir_name, dir.file_count))?;
                Ok(BsaDir {

                    name: dir_content.name
                        .map(FileId::StringId)
                        .unwrap_or(FileId::HashId(dir.name_hash)),
                    
                    files: dir_content.files.iter().map(|file| {
                        
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
impl Readable for DirRecord {}
impl Writable for DirRecord {
    fn size(&self) -> usize { size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        bin::write_struct(self, writer)
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
impl Readable for FileRecord {}
impl Writable for FileRecord {
    fn size(&self) -> usize { size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        bin::write_struct(self, writer)
    }
}


#[derive(Debug)]
pub struct FolderContentRecord {
    pub name: Option<BZString>,
    pub files: Vec<FileRecord>,
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
        let files = FileRecord::read_many0(reader, *file_count as usize)?;
        Ok(FolderContentRecord {
            name,
            files,
        })
    }
}
impl Writable for FolderContentRecord {
    fn size(&self) -> usize {
        self.files.size() + self.name.size()
    }
    fn write_here<W: Write>(&self, mut out: W) -> Result<()> {
        self.name.write_here(&mut out)?;
        self.files.write_here(&mut out)
    }
}

pub struct V10XWriter<T, AF: BitFlag, RDR> {
    phantom_t: PhantomData<T>,
    phantom_af: PhantomData<AF>,
    phantom_rdr: PhantomData<RDR>,
}

#[derive(Debug, Clone, Copy)]
pub struct V10XWriterOptions<AF: BitFlag> {
    pub archive_flags: BitFlags<AF>,
    pub file_flags: BitFlags<FileFlag>,
}
impl<AF: ToArchiveBitFlags> Default for V10XWriterOptions<AF> {
    fn default() -> Self {
        Self {
            archive_flags: BitFlags::empty()
                | AF::includes_file_names()
                | AF::includes_dir_names(),
            file_flags: BitFlags::empty(),
        }
    }
}
impl<AF: ToArchiveBitFlags> From<V10XWriterOptions<AF>> for V10XHeader<AF> {
    fn from(opts: V10XWriterOptions<AF>) -> Self { 
        let mut header = Self::default();
        header.archive_flags = opts.archive_flags;
        header.file_flags = opts.file_flags;
        header
    }
}
impl<T, AF, RDR> BsaWriter for V10XWriter<T, AF, RDR>
where
    T: Versioned,
    AF: ToArchiveBitFlags,
    RDR: From<DirRecord> + Writable + Sized + Copy
{
    type Options = V10XWriterOptions<AF>;
    fn write<D, W>(opts: Self::Options, dirs: D, mut out: W) -> Result<()>
    where
        D: IntoIterator<Item = BsaDirSource> + Copy,
        W: Write + Seek,
    {
        let version = T::version();
        version.write_here(&mut out)?;
        let mut header: V10XHeader<AF> = opts.into();

        let mut file_names: Vec<NullTerminated> = Vec::new();
        
        let includes_file_names = header.has(AF::includes_file_names());
        let includes_dir_names = header.has(AF::includes_dir_names());
        let is_compressed_by_default = header.has(AF::is_compressed_by_default());
        let embed_file_names = AF::embed_file_names()
            .map(|f| header.has(f))
            .unwrap_or(false);

        for dir in dirs {
            header.folder_count += 1;
            header.file_count += dir.files.len() as u32;
            
            if includes_dir_names {
                header.total_folder_name_length += (dir.name.len() as u32) + 1;
            }
            
            if includes_file_names {
                for file in dir.files {
                    let file_name = NullTerminated::from_str(&file.name.to_lowercase())?;
                    file_names.push(file_name);
                }
            }
        }

        header.total_file_name_length = bin::size_many(&file_names) as u32;

        header.write_here(&mut out)?;

        let drs = dirs.into_iter()
            .map(|dir| {
                let pos = out.stream_position()?;
                let dr = DirRecord {
                    name_hash: hash_v10x(&dir.name),
                    file_count: dir.files.len() as u32,
                    offset: 0,
                };
                RDR::from(dr).write_here(&mut out)?;
                Ok((pos, dr))
            })
            .collect::<Result<Vec<(u64, DirRecord)>>>()?;

        let fcrs = dirs.into_iter()
            .zip(drs)
            .map(|(dir, (dr_pos, mut dr))| {
                let fcr = FolderContentRecord {
                    name: if includes_dir_names {
                        Some(BZString::from(dir.name.to_lowercase()))
                    } else {
                        None
                    },
                    files: dir.files.iter()
                        .map(|file| FileRecord {
                            name_hash: hash_v10x(&file.name),
                            size: if file.compressed == Some(!is_compressed_by_default) {
                                0x40000000
                            } else {
                                0
                            },
                            offset: 0,
                        })
                        .collect(),
                };
                
                let pos = out.stream_position()?;
                dr.offset = pos as u32 + header.total_file_name_length;
                out.seek(SeekFrom::Start(dr_pos))?;
                RDR::from(dr).write_here(&mut out)?;
                out.seek(SeekFrom::Start(pos))?;
                fcr.write_here(&mut out)?;

                Ok((pos, fcr))
            })
            .collect::<Result<Vec<(u64, FolderContentRecord)>>>()?;

        file_names.write_here(&mut out)?;

        for (dir, (fcr_pos, mut fcr)) in dirs.into_iter().zip(fcrs) {

            for (mut file, mut fr) in dir.files.into_iter().zip(&mut fcr.files) {
                fr.offset = out.stream_position()? as u32;
                if embed_file_names {
                    let path = &format!("{}\\{}",
                        dir.name.replace("/", "\\"),
                        file.name.replace("/", "\\"));
                    NullTerminated::from_str(path)?
                        .write_here(&mut out)?;
                }
                fr.size |= if file.compressed.unwrap_or(is_compressed_by_default) {
                    let pos = out.stream_position()?;
                    // placeholder for size_orig
                    (0 as u32).write_here(&mut out)?;
                    let size_orig = T::compress(file.data, &mut out)? as u32;
                    let size_compressed = out.stream_position()? - pos;

                    let pos_tmp = out.stream_position()?;
                    out.seek(SeekFrom::Start(pos))?;
                    size_orig.write_here(&mut out)?;
                    out.seek(SeekFrom::Start(pos_tmp))?;

                    size_compressed as u32
                } else {
                    copy(&mut file.data, &mut out)? as u32
                }
            }
            
            let pos_tmp = out.stream_position()?;
            out.seek(SeekFrom::Start(fcr_pos))?;
            fcr.write_here(&mut out)?;
            out.seek(SeekFrom::Start(pos_tmp))?;
        }

        Ok(())
    }
}
