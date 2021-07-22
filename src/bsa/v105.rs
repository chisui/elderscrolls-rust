use std::io::{Read, Seek, SeekFrom, Result};
use std::mem::size_of;
use std::option::Option;
use std::collections::HashMap;
use bytemuck::{Zeroable, Pod};

use super::bzstring::NullTerminated;
use super::archive::{BsaDir, BsaFile, FileId};
pub use super::bin::{read_struct, Readable};
pub use super::hash;
pub use super::hash::Hash;
pub use super::v104::{ArchiveFlag, ArchiveFlag::CompressedArchive, FileFlag, Header, Has, RawHeader, FileRecord, BZString};


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct RawFolderRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub _padding_pre: u32,
    pub offset: u32,
    pub _padding_post: u32,
}
impl Readable for RawFolderRecord {
    fn read_here<R: Read + Seek>(reader: R, _: ()) -> Result<Self> {
        read_struct(reader)
    }
}

#[derive(Debug)]
pub struct FolderRecord {
    pub name_hash: Hash,
    pub name: Option<BZString>,
    pub files: Vec<FileRecord>,
}

#[derive(Debug)]
pub struct FolderContentRecord {
    pub name: Option<BZString>,
    pub file_records: Vec<FileRecord>,
}
impl Readable for FolderContentRecord {
    type ReadableArgs = (bool, u32);
    fn read_here<R: Read + Seek>(mut reader: R, (has_name, file_count): (bool, u32)) -> Result<FolderContentRecord> {
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

#[derive(Debug)]
pub struct FileNames(pub HashMap<Hash, BZString>);
impl Readable for FileNames {
    type ReadableArgs = Header;
    fn offset(header: Header) -> Option<u64> {
        FolderRecords::offset(header).map(|after_header| { 
            let foler_records_size = size_of::<RawFolderRecord>() as u64 * header.folder_count as u64;
            let foler_names_size = if header.has(ArchiveFlag::IncludeDirectoryNames) {
                header.total_folder_name_length as u64
                + header.folder_count as u64 // total_folder_name_length does not include size byte
            } else {
                0
            };
            after_header + foler_records_size + foler_names_size + header.file_count as u64 * size_of::<FileRecord>() as u64
        })
    }

    fn read_here<R: Read + Seek>(mut reader: R, header: Header) -> Result<FileNames> {
        Ok(FileNames(if header.has(ArchiveFlag::IncludeFileNames) {
            let names = NullTerminated::read_many(&mut reader, header.file_count as usize - 1, ())?;
            names.iter()
                .map(BZString::from)
                .map(|name| (Hash::from(&name), name.clone()))
                .collect()
        } else {
            HashMap::new()
        }))
    }
}

#[derive(Debug)]
pub struct FolderRecords(pub Vec<FolderRecord>);

impl Readable for FolderRecords {
    type ReadableArgs = Header;
    fn offset(_: Header) -> Option<u64> {
        Header::offset(()).map(|o| o + size_of::<Header>() as u64)
    }

    fn read_here<R: Read + Seek>(mut reader: R, header: Header) -> Result<FolderRecords> {
        let hasdir_name = header.has(ArchiveFlag::IncludeDirectoryNames);
        
        let raw_dirs = RawFolderRecord::read_many(&mut reader, header.folder_count as usize, ())?;
        let mut dir_contents = Vec::new();

        for dir in raw_dirs {
            reader.seek(SeekFrom::Start(dir.offset as u64- header.total_file_name_length as u64))?;
            let dir_content = FolderContentRecord::read(&mut reader, (hasdir_name, dir.file_count))?;
            dir_contents.push(FolderRecord {
                name_hash: dir.name_hash,
                name: dir_content.name,
                files: dir_content.file_records,
            });
        }
        Ok(FolderRecords(dir_contents))
    }
}

pub fn file_tree<R: Read + Seek>(mut reader: R, header: Header) -> Result<Vec<BsaDir>> {
    let FolderRecords(dirs) = FolderRecords::read(&mut reader, header)?;
    let FileNames(file_names) = FileNames::read(&mut reader, header)?;
    
    let files = dirs.iter().map(|dir| {
        BsaDir {

            name: dir.name
                .clone()
                .map(FileId::StringId)
                .unwrap_or(FileId::HashId(dir.name_hash)),
            
            files: dir.files.iter().map(|file| {
                
                let compressed = if header.has(CompressedArchive) {
                    !file.is_compression_bit_set()
                } else {
                    file.is_compression_bit_set()
                };

                BsaFile {
                    name: file_names.get(&file.name_hash)
                        .map(|n| n.clone())
                        .map(FileId::StringId)
                        .unwrap_or(FileId::HashId(file.name_hash)),
                    compressed,
                    offset: file.offset as u64,
                    size: file.size,
                }

            }).collect::<Vec<_>>(),
        }
    }).collect::<Vec<_>>();
    Ok(files)
}
