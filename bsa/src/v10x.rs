use std::io::{Read, Seek, SeekFrom, Result, Write, copy};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::size_of;
use std::str::{self, FromStr};
use std::fmt;
use bytemuck::{Pod, Zeroable};
use enumflags2::{bitflags, BitFlags, BitFlag};

use crate::bin::{self, DataSource, Fixed, Positioned, Readable, ReadableFixed, ReadableParam, VarSize, Writable, WritableFixed, derive_readable_via_pod, derive_writable_via_pod, read_struct};
use crate::compress::Compression;
use crate::str::{BZString, BString, ZString};
use crate::{EntryId, Hash};
use crate::version::{Version, Version10X, MagicNumber};
use crate::read::{self, Reader};
use crate::write::{self, Writer};


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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileFlag {
    Meshes = 0x1,
    Textures = 0x2,
    Menus = 0x4,
    Sounds = 0x8,
    Voices = 0x10,
    Shaders = 0x20,
    Trees = 0x40,
    Fonts = 0x80,
    Miscellaneous = 0x100,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct RawHeader {
    pub offset: u32,
    pub archive_flags: u32,
    pub dir_count: u32,
    pub file_count: u32,
    pub total_dir_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: u16,
    pub padding: u16,
}
impl Fixed for RawHeader {
    fn pos() -> usize { size_of::<(MagicNumber, Version10X)>() }
}
derive_var_size_via_size_of!(RawHeader);
derive_readable_fixed_via_default!(RawHeader);
derive_writable_fixed_via_default!(RawHeader);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeaderV10X<AF: BitFlag> {
    pub offset: u32,
    pub archive_flags: BitFlags<AF>,
    pub dir_count: u32,
    pub file_count: u32,
    pub total_dir_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: BitFlags<FileFlag>,
    pub padding: u16,
}
impl<AF: ToArchiveBitFlags + std::cmp::PartialEq> Eq for HeaderV10X<AF> {}
impl<AF: BitFlag> HeaderV10X<AF> {
    fn effective_total_dir_name_len(&self) -> usize {
        self.total_dir_name_length as usize
            + self.dir_count as usize // total_dir_name_length does not include size byte
    }
}
impl<AF: ToArchiveBitFlags> Default for HeaderV10X<AF> {
    fn default() -> Self {
        let mut h = Self::from(&RawHeader::zeroed());
        h.offset = size_of::<(MagicNumber, u32, RawHeader)>() as u32;
        h
    }
}
impl<AF: ToArchiveBitFlags> From<&RawHeader> for HeaderV10X<AF> {
    fn from(raw: &RawHeader) -> HeaderV10X<AF> {
        Self {
            offset: raw.offset,
            archive_flags: ToArchiveBitFlags::to_archive_bit_flags(raw.archive_flags),
            dir_count: raw.dir_count,
            file_count: raw.file_count,
            total_dir_name_length: raw.total_dir_name_length,
            total_file_name_length: raw.total_file_name_length,
            file_flags: BitFlags::from_bits_truncate(raw.file_flags),
            padding: raw.padding,
        }   
    }
}
impl<AF: ToArchiveBitFlags> From<&HeaderV10X<AF>> for RawHeader {
    fn from(h: &HeaderV10X<AF>) -> Self {
        Self {
            offset: h.offset,
            archive_flags: ToArchiveBitFlags::from_archive_bit_flags(h.archive_flags),
            dir_count: h.dir_count,
            file_count: h.file_count,
            total_dir_name_length: h.total_dir_name_length,
            total_file_name_length: h.total_file_name_length,
            file_flags: h.file_flags.bits(),
            padding: h.padding,
        }
    }
}
pub trait Has<T> {
    fn has(&self, t: T) -> bool;

    fn has_any<I: IntoIterator<Item = T> + Copy>(&self, flags: &I) -> bool {
        flags.into_iter()
            .any(|flag| self.has(flag))
    }
}
impl<AF: ToArchiveBitFlags> Has<AF> for HeaderV10X<AF> {
    fn has(&self, f: AF) -> bool {
        self.archive_flags.contains(f)
    }
}
impl<AF: ToArchiveBitFlags> Has<FileFlag> for HeaderV10X<AF> {
    fn has(&self, f: FileFlag) -> bool {
        self.file_flags.contains(f)
    }
}
impl<AF: ToArchiveBitFlags + fmt::Debug> bin::Fixed for HeaderV10X<AF> {
    fn pos() -> usize { RawHeader::pos() }
}
impl<AF: ToArchiveBitFlags + fmt::Debug> bin::VarSize for HeaderV10X<AF> {
    fn size(&self) -> usize { size_of::<RawHeader>() }
}
impl<AF: ToArchiveBitFlags + fmt::Debug> bin::ReadableFixed for HeaderV10X<AF> {
    fn read_fixed<R: Read + Seek>(reader: R) -> Result<Self> {
        let raw = RawHeader::read_fixed(reader)?;
        Ok(HeaderV10X::from(&raw))
    }
}
impl<AF: ToArchiveBitFlags> WritableFixed for HeaderV10X<AF> {
    fn write_fixed<W: Write + Seek>(&self, writer: W) -> Result<()> {
        RawHeader::from(self).write_fixed(writer)
    }
}

impl<AF: ToArchiveBitFlags + fmt::Debug> fmt::Display for HeaderV10X<AF> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "archive_flags:")?;
        for flag in self.archive_flags.iter() {
            writeln!(f, "    {:?}", flag)?;
        }
        writeln!(f, "dir_count: {}", self.dir_count)?;
        writeln!(f, "file_count: {}", self.file_count)?;
        writeln!(f, "total_dir_name_length: {}", self.total_dir_name_length)?;
        writeln!(f, "total_file_name_length: {}", self.total_file_name_length)?;
        writeln!(f, "file_flags:")?;
        for flag in self.file_flags.iter() {
            writeln!(f, "    {:?}", flag)?;
        }
        writeln!(f, "Direcotries: {}", self.dir_count)?;
        writeln!(f, "read::Files:   {}", self.file_count)
    }
}


pub struct ReaderV10X<R, T, C, AF: ToArchiveBitFlags, RDR> {
    pub(crate) reader: R,
    pub(crate) header: HeaderV10X<AF>,
    pub(crate) dirs: Option<Vec<read::Dir>>,
    phantom_t: PhantomData<T>,
    phantom_c: PhantomData<C>,
    phantom_rdr: PhantomData<RDR>,
}
impl<R, T, C, AF, RDR> ReaderV10X<R, T, C, AF, RDR>
where
    R: Read + Seek,
    T: Versioned,
    C: Compression,
    AF: ToArchiveBitFlags,
{
    fn offset_file_names(&self) -> usize {
        let dir_records_size = size_of::<RDR>() * self.header.dir_count as usize;
        let dir_names_size = if self.header.has(AF::includes_dir_names()) {
            self.header.effective_total_dir_name_len()
        } else {
            0
        };
        let file_records_size = self.header.file_count as usize * size_of::<FileRecord>();
        self.offset_after_header() + dir_records_size + dir_names_size + file_records_size
    }

    fn offset_after_header(&self) -> usize {
        size_of::<(MagicNumber, Version10X, RawHeader)>()
    }

    fn read_file_names(&mut self) -> Result<HashMap<Hash, ZString>> {
        self.reader.seek(SeekFrom::Start(self.offset_file_names() as u64))?;
        Ok(if self.header.has(AF::includes_file_names()) {
            let names = ZString::read_bin_many(&mut self.reader, self.header.file_count as usize)?;
            names.iter()
                .map(|name| (Hash::v10x(name.to_string().as_str()), name.clone()))
                .collect()
        } else {
            HashMap::new()
        })
    }

    fn read_dir(&mut self, file_names: &HashMap<Hash, ZString>, dir: &DirRecord) -> Result<read::Dir> {
        let has_dir_name = self.header.has(AF::includes_file_names());
        
        self.reader.seek(SeekFrom::Start(
            dir.offset as u64 - self.header.total_file_name_length as u64))?;
        let dir_content = DirContentRecord::read_with_param(&mut self.reader, (has_dir_name, dir.file_count))?;

        Ok(read::Dir {
            id: EntryId {
                hash: dir.name_hash,
                name: dir_content.name
                    .map(|n| n.to_string()),
            },
            files: dir_content.files.iter()
                .map(|file| self.to_file(&file_names, file))
                .collect(),
        })
    }

    fn to_file(&mut self, file_names: &HashMap<Hash, ZString>, file: &FileRecord) -> read::File {
        let compressed = if self.header.has(AF::is_compressed_by_default()) {
            !file.is_compression_bit_set()
        } else {
            file.is_compression_bit_set()
        };

        read::File {
            id: EntryId {
                hash: file.name_hash,
                name: file_names.get(&file.name_hash)
                    .map(|n| n.to_string()),
            },
            compressed,
            offset: file.offset as u64,
            size: file.real_size() as usize,
        }
    }
}
pub trait Versioned {
    fn version() -> Version10X;
}
impl<R, T, C, AF, RDR> Reader for ReaderV10X<R, T, C, AF, RDR>
where
    R: Read + Seek,
    T: Versioned,
    C: Compression,
    AF: ToArchiveBitFlags + fmt::Debug,
    RDR: Readable + Sized + Copy + fmt::Debug,
    DirRecord: From<RDR>,
{
    type Header = HeaderV10X<AF>;
    type In = R;

    fn read_bsa(mut reader: R) -> Result<Self> {
        let header = HeaderV10X::<AF>::read_fixed(&mut reader)?;
        Ok(Self {
            reader,
            header,
            dirs: None,
            phantom_t: PhantomData,
            phantom_c: PhantomData,
            phantom_rdr: PhantomData,
        })
    }


    fn header(&self) -> Self::Header {
        self.header
    }

    fn list(&mut self) -> Result<Vec<read::Dir>> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.to_vec())
        } else {
            self.reader.seek(SeekFrom::Start(self.offset_after_header() as u64))?;
            let raw_dirs = RDR::read_bin_many(&mut self.reader, self.header.dir_count as usize)?;
            let file_names = self.read_file_names()?;
            let dirs = raw_dirs.iter()
                .map(|dir| DirRecord::from(*dir) )
                .map(|dir| self.read_dir(&file_names, &dir))
                .collect::<Result<Vec<read::Dir>>>()?;
            self.dirs = Some(dirs.to_vec());
            Ok(dirs)
        } 
    }

    fn extract<W: Write>(&mut self, file: &read::File, mut writer: W) -> Result<()> {
        self.reader.seek(SeekFrom::Start(file.offset))?;
        
        // skip name field
        if self.header.has_any(&AF::embed_file_names()) {
            let name_len: u8 = read_struct(&mut self.reader)?;
            self.reader.seek(SeekFrom::Current(name_len as i64))?;
        }
        
        if file.compressed {
            // skip uncompressed size field
            self.reader.seek(SeekFrom::Current(size_of::<u32>() as i64))?;

            let sub_reader = (&mut self.reader).take(file.size as u64);
            C::uncompress(sub_reader, writer)?;
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
derive_var_size_via_size_of!(DirRecord);
derive_readable_via_pod!(DirRecord);
derive_writable_via_pod!(DirRecord);

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

    pub fn real_size(&self) -> u32 {
        let bit_mask = 0xffffffff ^ 0x40000000;
        self.size & bit_mask
    }
}
derive_var_size_via_size_of!(FileRecord);
derive_readable_via_pod!(FileRecord);
derive_writable_via_pod!(FileRecord);


#[derive(Debug)]
pub struct DirContentRecord {
    pub name: Option<BZString>,
    pub files: Vec<FileRecord>,
}
impl ReadableParam<(bool, u32)> for DirContentRecord {
    fn read_with_param<R: Read>(mut reader: R, (has_name, file_count): (bool, u32)) -> Result<Self> {
        let name = if has_name {
            let n = BZString::read_bin(&mut reader)?;
            Some(n)
        } else {
            None
        };
        let files = FileRecord::read_bin_many(reader, file_count as usize)?;
        Ok(DirContentRecord {
            name,
            files,
        })
    }
}
impl VarSize for DirContentRecord {
    fn size(&self) -> usize {
        self.files.size() + self.name.size()
    }
}
impl Writable for DirContentRecord {
    fn write<W: Write>(&self, mut out: W) -> Result<()> {
        self.name.write(&mut out)?;
        self.files.write(&mut out)
    }
}

struct FileNames {
    size: u32,
    values: Vec<ZString>,
}

pub struct WriterV10X<T, C, AF: BitFlag, RDR> {
    phantom_t: PhantomData<T>,
    phantom_c: PhantomData<C>,
    phantom_af: PhantomData<AF>,
    phantom_rdr: PhantomData<RDR>,
    pub archive_flags: BitFlags<AF>,
    pub file_flags: BitFlags<FileFlag>,
}
impl<T, C, AF, RDR> WriterV10X<T, C, AF, RDR>
where
    T: Versioned,
    C: Compression,
    AF: ToArchiveBitFlags,
    RDR: From<DirRecord> + Into<DirRecord> + Writable + Sized + Copy
{
    pub fn new<A, F>(archive_flags: A,file_flags: F) -> Self
    where
        A: IntoIterator<Item = AF>,
        F: IntoIterator<Item = FileFlag>,
    {
        Self {
            phantom_af: PhantomData,
            phantom_rdr: PhantomData,
            phantom_t: PhantomData,
            phantom_c: PhantomData,
            archive_flags: archive_flags.into_iter().collect(),
            file_flags: file_flags.into_iter().collect(),
        }
    }

    fn write_version<W: Write + Seek>(mut out: W) -> Result<()> {
        let version = Version::V10X(T::version());
        version.write_fixed(&mut out)
    }

    fn write_header<W, D>(&self, dirs: &Vec<write::Dir<D>>, out: W) -> Result<FileNames> 
    where W: Write + Seek,
    {
        let mut header = HeaderV10X::<AF>::from(self);
        let mut file_names = Vec::<ZString>::new();
        
        for dir in dirs.iter() {
            header.dir_count += 1;
            header.file_count += dir.files.len() as u32;
            
            if self.has(AF::includes_dir_names()) {
                header.total_dir_name_length += (dir.name.len() as u32) + 1;
            }
            
            if self.has(AF::includes_file_names()) {
                for file in dir.files.iter() {
                    let file_name = ZString::from_str(&file.name.to_lowercase())?;
                    file_names.push(file_name);
                }
            }
        }

        header.total_file_name_length = file_names.iter()
            .map(|n| n.size() as u32)
            .sum();

        header.write_fixed(out)?;
        
        Ok(FileNames {
            size: header.total_file_name_length,
            values: file_names
        })
    }

    fn write_dir_record<W, D>(dir: &write::Dir<D>, out: W) -> Result<Positioned<RDR>>
    where W: Write + Seek {
        let rec = DirRecord {
            name_hash: Hash::v10x(&dir.name),
            file_count: dir.files.len() as u32,
            offset: 0,
        };
        Positioned::new(RDR::from(rec), out)
    }

    fn write_dir_records<W, D>(dirs: &Vec<write::Dir<D>>, mut out: W) -> Result<Vec<Positioned<RDR>>>
    where W: Write + Seek {
        dirs.iter()
            .map(|dir| Self::write_dir_record(dir, &mut out))
            .collect()
    }

    fn write_dir_content_record<W, D>(&self, dir: &write::Dir<D>, out: W) -> Result<Positioned<DirContentRecord>>
    where W: Write + Seek {
        let name = if self.has(AF::includes_dir_names()) {
            let s = BZString::new(dir.name.to_lowercase())?;
            Some(s)
        } else {
            None
        };
        let files = dir.files.iter()
            .map(|file| FileRecord {
                name_hash: Hash::v10x(&file.name),
                size: if file.compressed == Some(!self.has(AF::is_compressed_by_default())) {
                    0x40000000
                } else {
                    0
                },
                offset: 0,
            })
            .collect();
        Positioned::new(DirContentRecord { name, files }, out)
    }

    fn write_dir_content_records<W, D>(
        &self,
        dirs: &Vec<write::Dir<D>>,
        dir_records: &mut Vec<Positioned<RDR>>,
        total_file_name_length: u32,
        mut out: W,
    ) -> Result<Vec<Positioned<DirContentRecord>>>
    where W: Write + Seek {
        dirs.iter().zip(dir_records)
            .map(|(dir, mut pdr)| {
                let fcr = self.write_dir_content_record(dir, &mut out)?;

                let mut dr: DirRecord = pdr.data.into();
                dr.offset = fcr.position as u32 + total_file_name_length;
                pdr.data = RDR::from(dr);
                pdr.update(&mut out)?;
                
                Ok(fcr)
            })
            .collect()
    }

    fn write_embeded_file_name<W>(dir: &String, file: &String, out: W) -> Result<()>
    where W: Write + Seek {
        let path = &format!("{}\\{}",
            dir.replace("/", "\\"),
            file.replace("/", "\\"));
        BString::from_str(path)?
            .write(out)
    }

    fn write_file_content<W, D>(&self, dir: &write::Dir<D>, file: &write::File<D>, mut out: W) -> Result<u64>
    where
        W: Write + Seek,
        D: DataSource,
    {
        let is_compressed_by_default = self.has(AF::is_compressed_by_default());
        if self.has_any(&AF::embed_file_names()) {
            Self::write_embeded_file_name(&dir.name, &file.name, &mut out)?;
        }
        let mut data_source = file.data.open()?;
        if file.compressed.unwrap_or(is_compressed_by_default) {
            let mut size_orig: Positioned<u32> = Positioned::new_empty(&mut out)?;
            size_orig.data = C::compress(data_source, &mut out)? as u32;
            size_orig.update(&mut out)?;
            
            Ok(out.stream_position()? - size_orig.position)
        } else {
            copy(&mut data_source, &mut out)
        }
    }

    fn write_file_contents<W, D: DataSource>(
        &self,
        dirs: &Vec<write::Dir<D>>,
        dir_content_records: &mut Vec<Positioned<DirContentRecord>>,
        mut out: W,
    ) -> Result<()>
    where W: Write + Seek {
        for (dir, pfcr) in dirs.iter().zip(dir_content_records) {
            
            for (file, mut fr) in dir.files.iter().zip(&mut pfcr.data.files) {
                fr.offset = out.stream_position()? as u32;
                fr.size |= self.write_file_content(dir, file, &mut out)? as u32;
            }
            pfcr.update(&mut out)?;
        }
        Ok(())
    }
   
}
impl<T, C, AF, RDR> Default for WriterV10X<T, C, AF, RDR>
where
    T: Versioned,
    C: Compression,
    AF: ToArchiveBitFlags,
    RDR: From<DirRecord> + Into<DirRecord> + Writable + Sized + Copy
{
    fn default() -> Self {
        Self::new([AF::includes_file_names(), AF::includes_dir_names()], [])
    }
}
impl<T, C, AF: ToArchiveBitFlags, RDR> From<&WriterV10X<T, C, AF, RDR>> for HeaderV10X<AF> {
    fn from(opts: &WriterV10X<T, C, AF, RDR>) -> Self { 
        let mut header = Self::default();
        header.archive_flags = opts.archive_flags;
        header.file_flags = opts.file_flags;
        header
    }
}
impl<T, C, AF: ToArchiveBitFlags, RDR> Has<AF> for WriterV10X<T, C, AF, RDR> {
    fn has(&self, f: AF) -> bool {
        self.archive_flags.contains(f)
    }
}
impl<T, C, AF: ToArchiveBitFlags, RDR> Has<FileFlag> for WriterV10X<T, C, AF, RDR> {
    fn has(&self, f: FileFlag) -> bool {
        self.file_flags.contains(f)
    }
}

impl<T, C, AF, RDR> Writer for WriterV10X<T, C, AF, RDR>
where
    T: Versioned,
    C: Compression,
    AF: ToArchiveBitFlags,
    RDR: From<DirRecord> + Into<DirRecord> + Writable + Sized + Copy + fmt::Debug
{
    fn write_bsa<DS, D, W>(&self, raw_dirs: DS, mut out: W) -> Result<()>
    where
        DS: IntoIterator<Item = write::Dir<D>>,
        D: DataSource,
        W: Write + Seek,
    {
        let dirs: Vec<write::Dir<D>> = raw_dirs.into_iter().collect();
        Self::write_version(&mut out)?;
        let file_names = self.write_header(&dirs, &mut out)?;
        let mut dir_records = Self::write_dir_records(&dirs, &mut out)?;
        let mut dir_content_records = self.write_dir_content_records(&dirs, &mut dir_records, file_names.size, &mut out)?;
        file_names.values.write(&mut out)?;
        self.write_file_contents(&dirs, &mut dir_content_records, &mut out)
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::v105::{HeaderV105, ArchiveFlagV105};
    use super::*;

    #[test]
    fn write_read_identity_header() -> Result<()> {
        let header_out = HeaderV105 {
            offset: 12,
            archive_flags: BitFlags::empty()
                | ArchiveFlagV105::CompressedArchive
                | ArchiveFlagV105::EmbedFileNames,
            dir_count: 13,
            file_count: 14,
            total_dir_name_length: 15,
            file_flags: BitFlags::empty()
                | FileFlag::Fonts
                | FileFlag::Menus,
            total_file_name_length: 16,
            padding: 13,
        };

        
        let mut out = Cursor::new(Vec::<u8>::new());
        header_out.write_fixed(&mut out)?;
        let mut input = Cursor::new(out.into_inner());
        let header_in = HeaderV105::read_fixed(&mut input)?;
        
        assert_eq!(header_out, header_in);
        
        Ok(())
    }
}
