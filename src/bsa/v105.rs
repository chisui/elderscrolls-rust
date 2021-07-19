
pub use super::{MagicNumber, Hash};
pub use super::v104::{ArchiveFlags, FileFlags, Header, FileRecord};


#[repr(C)]
#[derive(Debug)]
pub struct FolderRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub padding_pre: u32,
    pub offset: u32,
    pub padding_post: u32,
}
