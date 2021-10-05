use std::{
    io::{Read, Seek, SeekFrom, Result, Write, copy},
    collections::HashMap,
    marker::PhantomData,
    mem::size_of,
    str::{self, FromStr},
    fmt,
};
use bytemuck::{Pod, Zeroable};
use enumflags2::{bitflags, BitFlags, BitFlag};

use crate::{
    bin::{
        self, read_struct, Readable, Writable, Positioned, DataSource,
        derive_readable_via_pod, derive_writable_via_pod,
    },
    str::{BZString, BString, ZString},
    Hash,
    version::{Version, Version10X},
    magicnumber::MagicNumber,
    read::{BsaReader, BsaDir, BsaFile},
    write::{BsaWriter, BsaDirSource, BsaFileSource},
};


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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct V10XHeader<AF: BitFlag> {
    pub offset: u32,
    pub archive_flags: BitFlags<AF>,
    pub dir_count: u32,
    pub file_count: u32,
    pub total_dir_name_length: u32,
    pub total_file_name_length: u32,
    pub file_flags: BitFlags<FileFlag>,
    pub padding: u16,
}
impl<AF: BitFlag> V10XHeader<AF> {
    fn effective_total_dir_name_len(&self) -> usize {
        self.total_dir_name_length as usize
            + self.dir_count as usize // total_dir_name_length does not include size byte
    }
}
impl<AF: ToArchiveBitFlags + std::cmp::PartialEq> Eq for V10XHeader<AF> {}
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
            dir_count,
            file_count,
            total_dir_name_length,
            total_file_name_length,
            file_flags,
            padding,
        } = raw;
        Self {
            offset,
            archive_flags: ToArchiveBitFlags::to_archive_bit_flags(archive_flags),
            dir_count,
            file_count,
            total_dir_name_length,
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
            dir_count,
            file_count,
            total_dir_name_length,
            total_file_name_length,
            file_flags,
            padding,
        } = h;
        Self {
            offset,
            archive_flags: ToArchiveBitFlags::from_archive_bit_flags(archive_flags),
            dir_count,
            file_count,
            total_dir_name_length,
            total_file_name_length,
            file_flags: file_flags.bits(),
            padding,
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
        Some(size_of::<MagicNumber>() + size_of::<Version10X>())
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
        writeln!(f, "Files:   {}", self.file_count)
    }
}


pub struct V10XReader<R, T, AF: ToArchiveBitFlags, RDR> {
    pub reader: R,
    pub header: V10XHeader<AF>,
    pub dirs: Option<Vec<BsaDir>>,
    phantom_t: PhantomData<T>,
    phantom_rdr: PhantomData<RDR>,
}
impl<R, T, AF, RDR> V10XReader<R, T, AF, RDR>
where
    R: Read + Seek,
    T: Versioned,
    AF: ToArchiveBitFlags,
{
    pub(crate) fn read(mut reader: R) -> Result<Self> {
        let header = V10XHeader::<AF>::read0(&mut reader)?;
        Ok(V10XReader {
            reader,
            header,
            dirs: None,
            phantom_t: PhantomData,
            phantom_rdr: PhantomData,
        })
    }

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
        size_of::<MagicNumber>() + size_of::<Version10X>() + size_of::<RawHeader>()
    }

    fn read_file_names(&mut self) -> Result<HashMap<Hash, ZString>> {
        self.reader.seek(SeekFrom::Start(self.offset_file_names() as u64))?;
        Ok(if self.header.has(AF::includes_file_names()) {
            let names = ZString::read_many0(&mut self.reader, self.header.file_count as usize)?;
            names.iter()
                .map(|name| (Hash::v10x(name.to_string().as_str()), name.clone()))
                .collect()
        } else {
            HashMap::new()
        })
    }

    fn read_dir(&mut self, file_names: &HashMap<Hash, ZString>, dir: &DirRecord) -> Result<BsaDir> {
        let has_dir_name = self.header.has(AF::includes_file_names());
        
        self.reader.seek(SeekFrom::Start(
            dir.offset as u64 - self.header.total_file_name_length as u64))?;
        let dir_content = DirContentRecord::read(&mut self.reader, &(has_dir_name, dir.file_count))?;

        Ok(BsaDir {
            hash: dir.name_hash,
            name: dir_content.name
                .map(|n| n.to_string()),
            files: dir_content.files.iter()
                .map(|file| self.to_file(&file_names, file))
                .collect(),
        })
    }

    fn to_file(&mut self, file_names: &HashMap<Hash, ZString>, file: &FileRecord) -> BsaFile {
        let compressed = if self.header.has(AF::is_compressed_by_default()) {
            !file.is_compression_bit_set()
        } else {
            file.is_compression_bit_set()
        };

        BsaFile {
            hash: file.name_hash,
            name: file_names.get(&file.name_hash)
                .map(|n| n.to_string()),
            compressed,
            offset: file.offset as u64,
            size: file.real_size() as usize,
        }
    }
}
pub trait Versioned {
    fn version() -> Version10X;

    fn uncompress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64>;

    fn compress<R: Read, W: Write>(reader: R, writer: W) -> Result<u64>;
}
impl<R, T, AF, RDR> BsaReader for V10XReader<R, T, AF, RDR>
where
    R: Read + Seek,
    T: Versioned,
    AF: ToArchiveBitFlags + fmt::Debug,
    RDR: Readable<Arg = ()> + Sized + Copy + fmt::Debug,
    DirRecord: From<RDR>,
{
    type Header = V10XHeader<AF>;

    fn header(&self) -> Self::Header {
        self.header
    }

    fn list(&mut self) -> Result<Vec<BsaDir>> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.to_vec())
        } else {
            self.reader.seek(SeekFrom::Start(self.offset_after_header() as u64))?;
            let raw_dirs = RDR::read_many0(&mut self.reader, self.header.dir_count as usize)?;
            let file_names = self.read_file_names()?;
            let dirs = raw_dirs.iter()
                .map(|dir| DirRecord::from(*dir) )
                .map(|dir| self.read_dir(&file_names, &dir))
                .collect::<Result<Vec<BsaDir>>>()?;
            self.dirs = Some(dirs.to_vec());
            Ok(dirs)
        } 
    }

    fn extract<W: Write>(&mut self, file: &BsaFile, mut writer: W) -> Result<()> {
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
derive_readable_via_pod!(FileRecord);
derive_writable_via_pod!(FileRecord);


#[derive(Debug)]
pub struct DirContentRecord {
    pub name: Option<BZString>,
    pub files: Vec<FileRecord>,
}
impl Readable for DirContentRecord {
    type Arg = (bool, u32);
    fn read_here<R: Read + Seek>(mut reader: R, (has_name, file_count): &(bool, u32)) -> Result<DirContentRecord> {
        let name = if *has_name {
            let n = BZString::read0(&mut reader)?;
            Some(n)
        } else {
            None
        };
        let files = FileRecord::read_many0(reader, *file_count as usize)?;
        Ok(DirContentRecord {
            name,
            files,
        })
    }
}
impl Writable for DirContentRecord {
    fn size(&self) -> usize {
        self.files.size() + self.name.size()
    }
    fn write_here<W: Write>(&self, mut out: W) -> Result<()> {
        self.name.write_here(&mut out)?;
        self.files.write_here(&mut out)
    }
}

struct FileNames {
    size: u32,
    values: Vec<ZString>,
}

pub struct V10XWriter<T, AF: BitFlag, RDR> {
    phantom_t: PhantomData<T>,
    phantom_af: PhantomData<AF>,
    phantom_rdr: PhantomData<RDR>,
}

impl<T, AF, RDR> V10XWriter<T, AF, RDR>
where
    T: Versioned,
    AF: ToArchiveBitFlags,
    RDR: From<DirRecord> + Into<DirRecord> + Writable + Sized + Copy
{
    fn write_version<W: Write>(mut out: W) -> Result<()> {
        let version = Version::V10X(T::version());
        version.write_here(&mut out)
    }

    fn write_header<W, D>(opts: V10XWriterOptions<AF>, dirs: &Vec<BsaDirSource<D>>, mut out: W) -> Result<FileNames> 
    where W: Write + Seek,
    {
        let mut header: V10XHeader<AF> = opts.into();

        let mut file_names: Vec<ZString> = Vec::new();
        
        let includes_file_names = opts.has(AF::includes_file_names());
        let includes_dir_names = opts.has(AF::includes_dir_names());
        
        for dir in dirs.iter() {
            header.dir_count += 1;
            header.file_count += dir.files.len() as u32;
            
            if includes_dir_names {
                header.total_dir_name_length += (dir.name.len() as u32) + 1;
            }
            
            if includes_file_names {
                for file in dir.files.iter() {
                    let file_name = ZString::from_str(&file.name.to_lowercase())?;
                    file_names.push(file_name);
                }
            }
        }

        header.total_file_name_length = file_names.iter()
            .map(|n| n.size() as u32)
            .sum::<u32>();

        header.write_here(&mut out)?;
        
        Ok(FileNames {
            size: header.total_file_name_length,
            values: file_names
        })
    }

    fn write_dir_record<W, D>(dir: &BsaDirSource<D>, out: W) -> Result<Positioned<RDR>>
    where W: Write + Seek {
        let rec = DirRecord {
            name_hash: Hash::v10x(&dir.name),
            file_count: dir.files.len() as u32,
            offset: 0,
        };
        Positioned::new(RDR::from(rec), out)
    }

    fn write_dir_records<W, D>(dirs: &Vec<BsaDirSource<D>>, mut out: W) -> Result<Vec<Positioned<RDR>>>
    where W: Write + Seek {
        dirs.iter()
            .map(|dir| Self::write_dir_record(dir, &mut out))
            .collect()
    }

    fn write_dir_content_record<W, D>(opts: V10XWriterOptions<AF>, dir: &BsaDirSource<D>, out: W) -> Result<Positioned<DirContentRecord>>
    where W: Write + Seek {
        let name = if opts.has(AF::includes_dir_names()) {
            let s = BZString::new(dir.name.to_lowercase())?;
            Some(s)
        } else {
            None
        };
        let files = dir.files.iter()
            .map(|file| FileRecord {
                name_hash: Hash::v10x(&file.name),
                size: if file.compressed == Some(!opts.has(AF::is_compressed_by_default())) {
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
        opts: V10XWriterOptions<AF>,
        dirs: &Vec<BsaDirSource<D>>,
        dir_records: &mut Vec<Positioned<RDR>>,
        total_file_name_length: u32,
        mut out: W,
    ) -> Result<Vec<Positioned<DirContentRecord>>>
    where W: Write + Seek {
        dirs.iter().zip(dir_records)
            .map(|(dir, mut pdr)| {
                let fcr = Self::write_dir_content_record(opts, dir, &mut out)?;

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
            .write_here(out)
    }

    fn write_file_content<W, D>(opts: V10XWriterOptions<AF>, dir: &BsaDirSource<D>, file: &BsaFileSource<D>, mut out: W) -> Result<u64>
    where
        W: Write + Seek,
        D: DataSource,
    {
        let is_compressed_by_default = opts.has(AF::is_compressed_by_default());
        if opts.has_any(&AF::embed_file_names()) {
            Self::write_embeded_file_name(&dir.name, &file.name, &mut out)?;
        }
        let mut data_source = file.data.open()?;
        if file.compressed.unwrap_or(is_compressed_by_default) {
            let mut size_orig: Positioned<u32> = Positioned::new_empty(&mut out)?;
            size_orig.data = T::compress(data_source, &mut out)? as u32;
            size_orig.update(&mut out)?;
            
            Ok(out.stream_position()? - size_orig.position)
        } else {
            copy(&mut data_source, &mut out)
        }
    }

    fn write_file_contents<W, D: DataSource>(
        opts: V10XWriterOptions<AF>,
        dirs: &Vec<BsaDirSource<D>>,
        dir_content_records: &mut Vec<Positioned<DirContentRecord>>,
        mut out: W,
    ) -> Result<()>
    where W: Write + Seek {
        for (dir, pfcr) in dirs.iter().zip(dir_content_records) {
            
            for (file, mut fr) in dir.files.iter().zip(&mut pfcr.data.files) {
                fr.offset = out.stream_position()? as u32;
                fr.size |= Self::write_file_content(opts, dir, file, &mut out)? as u32;
            }
            pfcr.update(&mut out)?;
        }
        Ok(())
    }
   
}

#[derive(Debug, Clone, Copy)]
pub struct V10XWriterOptions<AF: BitFlag> {
    pub archive_flags: BitFlags<AF>,
    pub file_flags: BitFlags<FileFlag>,
}
impl<AF: ToArchiveBitFlags> Default for V10XWriterOptions<AF> {
    fn default() -> Self {
        let mut archive_flags = BitFlags::empty();
        archive_flags |= AF::includes_file_names();
        archive_flags |= AF::includes_dir_names();
        Self {
            archive_flags,
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
impl<AF: ToArchiveBitFlags> Has<AF> for V10XWriterOptions<AF> {
    fn has(&self, f: AF) -> bool {
        self.archive_flags.contains(f)
    }
}
impl<AF: ToArchiveBitFlags> Has<FileFlag> for V10XWriterOptions<AF> {
    fn has(&self, f: FileFlag) -> bool {
        self.file_flags.contains(f)
    }
}

impl<T, AF, RDR> BsaWriter for V10XWriter<T, AF, RDR>
where
    T: Versioned,
    AF: ToArchiveBitFlags,
    RDR: From<DirRecord> + Into<DirRecord> + Writable + Sized + Copy + fmt::Debug
{
    type Options = V10XWriterOptions<AF>;
    fn write_bsa<DS, D, W>(opts: Self::Options, raw_dirs: DS, mut out: W) -> Result<()>
    where
        DS: IntoIterator<Item = BsaDirSource<D>>,
        D: DataSource,
        W: Write + Seek,
    {
        let dirs: Vec<BsaDirSource<D>> = raw_dirs.into_iter().collect();
        Self::write_version(&mut out)?;
        let file_names = Self::write_header(opts, &dirs, &mut out)?;
        let mut dir_records = Self::write_dir_records(&dirs, &mut out)?;
        let mut dir_content_records = Self::write_dir_content_records(opts, &dirs, &mut dir_records, file_names.size, &mut out)?;
        file_names.values.write_here(&mut out)?;
        Self::write_file_contents(opts, &dirs, &mut dir_content_records, &mut out)
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::v105;
    use super::*;

    #[test]
    fn write_read_identity_header() -> Result<()> {
        let header_out = v105::Header {
            offset: 12,
            archive_flags: BitFlags::empty()
                | v105::ArchiveFlag::CompressedArchive
                | v105::ArchiveFlag::EmbedFileNames,
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
        header_out.write_here(&mut out)?;
        let mut input = Cursor::new(out.into_inner());
        let header_in = v105::Header::read_here0(&mut input)?;
        
        assert_eq!(header_out, header_in);
        
        Ok(())
    }
}
