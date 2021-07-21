use std::io::{Read, Result};
use bytemuck::{Zeroable, Pod};

pub use super::bin::read_struct;
pub use super::hash::Hash;
pub use super::v104::{ArchiveFlags, FileFlags, Header, RawHeader, FileRecord, BZString};


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FolderRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub _padding_pre: u32,
    pub offset: u32,
    pub _padding_post: u32,
}

#[derive(Debug)]
pub struct FolderContentRecord {
    pub name: Option<BZString>,
    pub file_records: Vec<FileRecord>,
}
impl FolderContentRecord {
    pub fn read<R: Read>(has_name: bool, file_count: u32, mut reader: R) -> Result<FolderContentRecord> {
        let name = if has_name {
            let n = BZString::read(&mut reader)?;
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
