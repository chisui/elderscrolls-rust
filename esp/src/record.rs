use std::mem::size_of;
use std::io::{Read, Seek, SeekFrom, Result};
use std::{fmt, str};

use bytemuck::{Pod, Zeroable};

use crate::bin::{Readable, read_struct};


#[derive(Debug, Clone)]
pub struct EspReader<R> {
    reader: R,
    top_level_entries: Option<Vec<Entry>>,
}
impl<R: Read + Seek> EspReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            top_level_entries: None,
        }
    }

    pub fn top_level_entries(&mut self) -> Result<&Vec<Entry>> {
        if None == self.top_level_entries {
            let size = self.reader.stream_len()?;
            let e = self.read_entris(0, size)?;
            self.top_level_entries = Some(e.to_vec());
        }
        Ok(&self.top_level_entries.as_ref()
            .unwrap())
    }

    fn read_entris(&mut self, start: u64, size: u64) -> Result<Vec<Entry>> {
        let end = start + size;
        let mut pos = self.reader.seek(SeekFrom::Start(start))?;
        let mut entries = Vec::new();
        while pos < end {
            let entry = Entry::read(&mut self.reader)?;
            self.reader.seek(SeekFrom::Current(entry.data_size() as i64))?;
            entries.push(entry);
            pos = self.reader.stream_position()?;
        }
        Ok(entries)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct Label([u8; 4]);
impl Label {
    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }
}
impl<R: Read> Readable<R> for Label {
    fn read(reader: &mut R) -> Result<Self> {
        read_struct(reader)
    }
}
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(s) => f.write_str(s),
            None => write!(f, "{:?}", self.0),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct VersionControlInfo {
    last_user: u8,
    current_user: u8,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct Timestamp(u16);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entry {
    Record(Record),
    Group(Group),
}
impl Entry {
    pub fn data_size(&self) -> u64 {
        match self {
            Entry::Record(r) => r.data_size as u64,
            Entry::Group(g) => g.data_size as u64,
        }
    }
}
impl<R: Read + Seek> Readable<R> for Entry {
    fn read(reader: &mut R) -> Result<Self> {
        let l= read_struct(reader)?;
        if l == Label(*b"GRUP") {
            Group::read(reader)
                .map(Entry::Group)
        } else {
            Record::read_after_label(reader, l)
                .map(Entry::Record)
        }
    }    
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Record {
    position: u64,
    data_size: u32,
    pub record_type: Label,
    pub flags: u32,
    pub id: u32,
    pub timestamp: Timestamp,
    pub version_control_info: VersionControlInfo,
    pub internal_version: u16,
    fields: Option<Vec<Field>>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct RecordHeader {
    data_size: u32,
    flags: u32,
    id: u32,
    timestamp: Timestamp,
    version_control_info: VersionControlInfo,
    internal_version: u16,
    _unknown: u16,
}
impl Record {
    fn read_after_label<R: Read + Seek>(reader: &mut R, record_type: Label) -> Result<Self> {
        let position = reader.stream_position()? - size_of::<Label>() as u64;
        let RecordHeader {
            data_size,
            flags,
            id,
            timestamp,
            version_control_info,
            internal_version,
            ..
        } = read_struct(reader)?;

        Ok(Record {
            data_size,
            position,

            record_type,
            flags,
            id,
            timestamp,
            version_control_info,
            internal_version,
            fields: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group {
    data_size: u32,
    position: u64,
    pub group_info: GroupInfo,
    pub timestamp: Timestamp,
    pub version_control_info: VersionControlInfo,
    members: Option<Vec<Entry>>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,  Zeroable, Pod)]
pub struct GroupInfo {
    pub label: Label,
    pub group_type: u32,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct GroupHeader {
    data_size: u32,
    group_info: GroupInfo,
    timestamp: Timestamp,
    version_control_info: VersionControlInfo,
    _unknown: u32,
}
const TOTAL_GROUP_HEADER_SIZE: usize
    = size_of::<Label>()
    + size_of::<GroupHeader>();
impl<R: Read + Seek> Readable<R> for Group {
    fn read(reader: &mut R) -> Result<Self> {
        let position = reader.stream_position()? - size_of::<Label>() as u64;
        let GroupHeader {
            data_size,
            group_info,
            timestamp,
            version_control_info,
            ..
        } = read_struct(reader)?;

        Ok(Group {
            data_size: data_size - TOTAL_GROUP_HEADER_SIZE as u32,
            position,

            group_info,
            timestamp,
            version_control_info,
            members: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field {
    pub field_type: Label,
    data_position: u64,
    data_size: u32,
    data: Option<Vec<u8>>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct FieldHeader {
    field_type: Label,
    data_size: u16,
}
impl<R: Read + Seek> Readable<R> for Field {
    fn read(reader: &mut R) -> Result<Self> {
        let header0: FieldHeader = read_struct(reader)?;
        let mut field_type = header0.field_type;
        let mut data_size = header0.data_size as u32;
        if field_type == Label(*b"XXXX") {
            data_size = read_struct(reader)?;
            let header1: FieldHeader = read_struct(reader)?;
            field_type = header1.field_type;
        };
        let data_position = reader.stream_position()?;
        Ok(Field {
            field_type,
            data_position,
            data_size,
            data: None,
        })
    }
}
