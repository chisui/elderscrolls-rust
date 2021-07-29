use std::io::{Read, Write, Seek, Result, copy};
use std::fmt;
use bytemuck::{Zeroable, Pod};


pub use super::bin::{read_struct, Readable};
pub use super::version::Version;
pub use super::hash::{hash_v10x, Hash};
pub use super::v10x::{V10X, Versioned};
pub use super::v10x;
pub use super::v104::{ArchiveFlag, Header, BZString};


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct RawDirRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub _padding_pre: u32,
    pub offset: u32,
    pub _padding_post: u32,
}
impl Readable for RawDirRecord {
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<Self> {
        read_struct(reader)
    }
}
impl From<RawDirRecord> for v10x::DirRecord {
    fn from(rec: RawDirRecord) -> Self {
        Self {
            name_hash: Hash::from(rec.name_hash),
            file_count: rec.file_count,
            offset: rec.offset,
        }
    }
}

pub enum V105T{}
impl Versioned for V105T {
    fn version() -> Version { Version::V105 }
    fn fmt_version(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BSA v105 file, format used by: TES V: Skyrim Special Edition")
    }

    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = lz4::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }
}

pub type V105 = V10X<V105T, ArchiveFlag, RawDirRecord>;
