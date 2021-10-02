use std::{
    io::{Read, Seek, Result},
    mem::size_of,
};
use bytemuck::{Pod, Zeroable};

use super::{
    bin::{read_struct, Readable},
    magicnumber::MagicNumber,
};
pub use super::str::{BZString, ZString};


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Header {
    pub offset_hash_table: u32,
    pub num_files: u32,
}
impl Readable for Header {
    fn offset(_: &()) -> Option<usize> {
        Some(size_of::<MagicNumber>())
    }
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<Self> {
        read_struct(reader)
    }
}

pub struct V001(pub Header);

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct FileRecord {
    pub size: u32,
    pub offset: u32,
}
impl Readable for FileRecord {
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<Self> {
        read_struct(reader)
    }
}

#[derive(Debug)]
pub struct FileRecords(pub Vec<FileRecord>);
impl Readable for FileRecords {
    type ReadableArgs = Header;
    fn offset(_: &Header) -> Option<usize> {
        Header::offset0().map(|h| h + size_of::<Header>())
    }
    fn read_here<R: Read + Seek>(reader: R, header: &Header) -> Result<Self> {
        FileRecord::read_many0(reader, header.num_files as usize)
            .map(FileRecords)
    }
}

#[derive(Debug)]
pub struct FileNames(pub Vec<ZString>);
impl Readable for FileNames {
    type ReadableArgs = Header;
    fn offset(header: &Header) -> Option<usize> {
        FileRecords::offset(header)
            .map(|fr| fr + (header.num_files as usize) * size_of::<FileRecord>())
    }
    fn read_here<R: Read + Seek>(reader: R, header: &Header) -> Result<Self> {
        ZString::read_many0(reader, header.num_files as usize)
            .map(FileNames)
    }
}


#[derive(Debug)]
pub struct FileNameHashes(pub Vec<u64>);
impl Readable for FileNameHashes {
    type ReadableArgs = Header;
    fn offset(header: &Header) -> Option<usize> {
        Header::offset0().map(|h| (header.offset_hash_table as usize) - h)
    }
    fn read_here<R: Read + Seek>(reader: R, header: &Header) -> Result<Self> {
        u64::read_many0(reader, header.num_files as usize)
            .map(FileNameHashes)
    }
}
