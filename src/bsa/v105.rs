use bytemuck::{Zeroable, Pod};
pub use super::hash::Hash;
pub use super::v104::{ArchiveFlags, FileFlags, Header, RawHeader, FileRecord};


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
    pub name: Option<String>,
    pub file_records: Vec<FileRecord>,
}
impl FolderContentRecord {
    pub fn new(name: Option<String>) -> FolderContentRecord {
        FolderContentRecord {
            name,
            file_records: Vec::new(),
        }
    }
}
