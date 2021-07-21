use std::io::{Read, Result};
use std::collections::HashMap;
use bytemuck::{Zeroable, Pod};

use super::bzstring::NullTerminated;
pub use super::bin::{read_struct, Readable};
pub use super::hash::Hash;
pub use super::v104::{ArchiveFlags, FileFlag, Header, RawHeader, FileRecord, BZString};


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
    fn read<R: Read>(reader: R, _: &()) -> Result<Self> {
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
    fn read<R: Read>(mut reader: R, (has_name, file_count): &(bool, u32)) -> Result<FolderContentRecord> {
        let name = if *has_name {
            let n = BZString::read(&mut reader, &())?;
            Some(n)
        } else {
            None
        };
        let mut file_records = Vec::with_capacity(*file_count as usize);
        for _ in 0..*file_count {
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
pub struct FileNames(HashMap<Hash, BZString>);
impl FileNames {
    pub fn empty() -> Self {
        FileNames(HashMap::new())
    }
}
impl Readable for FileNames {
    type ReadableArgs = u32;
    fn read<R: Read>(mut reader: R, file_count: &u32) -> Result<FileNames> {
        let mut file_names: HashMap<Hash, BZString> = HashMap::with_capacity(*file_count as usize);
        for _ in 0..*file_count {
            let name_nt = NullTerminated::read(&mut reader, &())?;
            let name = BZString::from(name_nt);
            file_names.insert(Hash::from(&name), name);
        }
        Ok(FileNames(file_names))
    }
}
