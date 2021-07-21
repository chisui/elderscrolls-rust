use std::io::{Read, Seek, SeekFrom, Result, Error, ErrorKind};
use std::mem::size_of;
use std::collections::HashMap;
use bytemuck::{Zeroable, Pod};

use super::bzstring::NullTerminated;
use super::version::{Version, MagicNumber};
pub use super::bin::{read_struct, Readable};
pub use super::hash::Hash;
pub use super::v104::{ArchiveFlag, FileFlag, Header, RawHeader, FileRecord, BZString};


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FolderRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub _padding_pre: u32,
    pub offset: u32,
    pub _padding_post: u32,
}
impl Readable for FolderRecord {
    type ReadableArgs = ();
    fn read<R: Read + Seek>(reader: R, _: ()) -> Result<Self> {
        read_struct(reader)
    }
}

#[derive(Debug)]
pub struct FolderContentRecord {
    pub name: Option<BZString>,
    pub file_records: Vec<FileRecord>,
}
impl Readable for FolderContentRecord {
    type ReadableArgs = (bool, u32);
    fn read<R: Read + Seek>(mut reader: R, (has_name, file_count): (bool, u32)) -> Result<FolderContentRecord> {
        let name = if has_name {
            let n = BZString::read(&mut reader, ())?;
            Some(n)
        } else {
            None
        };
        let mut file_records = Vec::with_capacity(file_count as usize);
        for _ in 0..file_count {
            let file = read_struct(&mut reader)?;
            file_records.push(file);
        }
        Ok(FolderContentRecord {
            name,
            file_records,
        })
    }
}

pub struct Bsa<R: Read + Seek> {
    reader: R,
    pub header: Header,
    _dirs: Option<Vec<FolderRecord>>,
    _dir_contents: Option<Vec<FolderContentRecord>>,
    _file_names: Option<HashMap<Hash, BZString>>,
}

impl<R: Read + Seek> Bsa<R> {
    pub fn open(mut reader: R) -> Result<Bsa<R>> {
        reader.seek(SeekFrom::Start(Bsa::<R>::offset_header()))?;
        let header = Header::read(&mut reader, ())?;
        Ok(Bsa {
            reader,
            header,
            _dirs: None,
            _dir_contents: None,
            _file_names: None,
        })
    }

    pub fn offset_header() -> u64 {
        size_of::<MagicNumber>() as u64 + size_of::<Version>() as u64
    }

    pub fn offset_dirs() -> u64 {
        Bsa::<R>::offset_header() + size_of::<Header>() as u64
    }
    pub fn dirs(&mut self) -> Result<&Vec<FolderRecord>> {
        if self._dirs.is_none() {
            self._dirs = load_dirs(&self.header, &mut self.reader)
                .map(Option::Some)?;
        }
        self._dirs.as_ref().ok_or(Error::new(ErrorKind::InvalidData, "could not load Folder Records"))
    }

    pub fn offset_dir_contents(&self) -> Result<u64> {
        Ok(Bsa::<R>::offset_dirs() + size_of::<FolderRecord>() as u64 * self.header.folder_count as u64)
    }
    pub fn dir_contents<'s>(&'s mut self) -> Result<&'s Vec<FolderContentRecord>> {
        if self._dirs.is_none() {
            let offset = self.offset_dir_contents()?;
            let mut reader = &mut self.reader;
            reader.seek(SeekFrom::Start(offset))?;

            let has_dir_name = self.header.has_archive_flag(ArchiveFlag::IncludeDirectoryNames);
            let mut dir_contents = Vec::new();

            if self._dirs.is_none() {
                self._dirs = load_dirs(&self.header, &mut reader)
                    .map(Option::Some)?;
            }

            for dir in self._dirs.as_ref().unwrap() {
                let dir_content = FolderContentRecord::read(&mut reader, (has_dir_name, dir.file_count))?;
                dir_contents.push(dir_content);
            }
            self._dir_contents = Some(dir_contents);
        }
        self._dir_contents.as_ref().ok_or(Error::new(ErrorKind::InvalidData, "could not load Folder contents"))
    }

    pub fn offset_file_names(&mut self) -> Result<u64> {
        let offset = self.offset_dir_contents()?;
        let foler_names_size = if self.header.has_archive_flag(ArchiveFlag::IncludeDirectoryNames) {
            self.header.total_folder_name_length as u64
            + self.header.folder_count as u64 // total_folder_name_length does not include size byte
        } else {
            0
        };
        Ok(offset + foler_names_size + self.header.file_count as u64 * size_of::<FileRecord>() as u64)
    }
    pub fn file_names<'s>(&'s mut self) -> Result<&'s HashMap<Hash, BZString>> {
        if self._file_names.is_none() {
            
            self._file_names = Some(if self.header.has_archive_flag(ArchiveFlag::IncludeFileNames) {
                let offset = self.offset_file_names()?;
                let mut reader = &mut self.reader;
                reader.seek(SeekFrom::Start(offset))?;
                let mut file_names: HashMap<Hash, BZString> = HashMap::with_capacity(self.header.file_count as usize);
                for _ in 0..self.header.file_count {
                    NullTerminated::read(&mut reader, ())
                        .map(BZString::from)
                        .map(|name| file_names.insert(Hash::from(&name), name))?;
                }
                file_names
            } else {
                HashMap::new()
            })
        }
        self._file_names.as_ref().ok_or(Error::new(ErrorKind::InvalidData, "could not load file names"))
    }
}

fn load_dirs<R: Read + Seek>(header: &Header, mut reader: R) -> Result<Vec<FolderRecord>> {
    reader.seek(SeekFrom::Start(Bsa::<R>::offset_dirs()))?;
    FolderRecord::read_many(&mut reader, header.folder_count as usize, ())
}
