use std::mem::size_of;
use std::io::{Read, Seek, SeekFrom, Result};
use std::{fmt, str};

use bytemuck::{Pod, Zeroable};

use crate::bin::{Readable, ReadStructExt};


pub fn entries<'a, R>(reader: &'a mut R) -> Result<Entries<'a, R>>
where R: Read + Seek {
    let end = reader.stream_len()?;
    Ok(Entries {
        reader,
        current: 0,
        end,
    })
}

pub struct Entries<'a, R> {
    reader: &'a mut R,
    current: u64,
    end: u64,
}
impl<'a, R> Entries<'a, R>
where R: Read + Seek {
    pub fn next_entry(&'a mut self) -> Result<Option<Entry<'a, R>>> {
        if self.current < self.end {
            self.reader.seek(SeekFrom::Start(self.current))?;
            let (entry, next) = read_entry(self.reader)?;
            self.current = next;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
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
impl<'a, R> Readable<'a, Label> for R
where R: Read {
    fn read_val(&mut self) -> Result<Label> {
        self.read_struct()
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entry<'a, R> {
    Record(Record<'a, R>),
    Group(Group<'a, R>),
}
impl<'a, R> Entry<'a, R> {
    pub fn data_size(&self) -> u64 {
        match self {
            Entry::Record(r) => r.data_size as u64,
            Entry::Group(g) => g.data_size as u64,
        }
    }
}

fn read_entry<'a, R>(reader: &'a mut R) -> Result<(Entry<'a, R>, u64)>
where R: Read + Seek {
    let l= reader.read_struct()?;
    if l == Label(*b"GRUP") {
        let (g, next) = read_group(reader)?;
        Ok((Entry::Group(g), next))
    } else {
        let (r, next) = read_record_after_label(reader, l)?;
        Ok((Entry::Record(r), next))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Record<'a, R> {
    reader: &'a mut R,
    position: u64,
    data_size: u32,
    pub record_type: Label,
    pub flags: u32,
    pub id: u32,
    pub timestamp: Timestamp,
    pub version_control_info: VersionControlInfo,
    pub internal_version: u16,
}

impl<'a, R> Record<'a, R>
where R: Read + Seek {
    pub fn fields(&'a self) -> Fields<'a, R> {
        let current = self.position + TOTAL_Record_HEADER_SIZE as u64;
        Fields {
            reader: self.reader,
            current,
            end: current + self.data_size as u64,
        }
    }
}

pub struct Fields<'a, R> {
    reader: &'a mut R,
    current: u64,
    end: u64,
}
impl<'a, R> Fields<'a, R>
where R: Read + Seek {
    pub fn next_field(&'a mut self) -> Result<Option<Field<'a, R>>> {
        if self.current < self.end {
            self.reader.seek(SeekFrom::Start(self.current))?;
            let (field, next) = read_field(self.reader)?;
            self.current = next;
            Ok(Some(field))
        } else {
            Ok(None)
        }
    }
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
fn read_record_after_label<'a, R>(reader: &'a mut R, record_type: Label) -> Result<(Record<'a, R>, u64)>
where R: Read + Seek {
    let position = reader.stream_position()? - size_of::<Label>() as u64;
    let RecordHeader {
        data_size,
        flags,
        id,
        timestamp,
        version_control_info,
        internal_version,
        ..
    } = reader.read_struct()?;
    let next = position + data_size as u64;

    Ok((Record {
        reader,
        data_size,
        position,

        record_type,
        flags,
        id,
        timestamp,
        version_control_info,
        internal_version,
    }, next))
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group<'a, R> {
    reader: &'a mut R,
    data_size: u32,
    position: u64,
    pub group_info: GroupInfo,
    pub timestamp: Timestamp,
    pub version_control_info: VersionControlInfo,
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

const TOTAL_Record_HEADER_SIZE: usize
    = size_of::<Label>()
    + size_of::<RecordHeader>();

fn read_group<'a, R>(reader: &'a mut R) -> Result<(Group<'a, R>, u64)>
where R: Read + Seek {
    let position = reader.stream_position()? - size_of::<Label>() as u64;
    let GroupHeader {
        data_size,
        group_info,
        timestamp,
        version_control_info,
        ..
    } = reader.read_struct()?;
    let effective_data_size = data_size - TOTAL_GROUP_HEADER_SIZE as u32;
    let next = position - effective_data_size as u64;
    Ok((Group {
        reader,
        data_size: effective_data_size,
        position,

        group_info,
        timestamp,
        version_control_info,
    }, next))
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field<'a, R> {
    reader: &'a mut R,
    pub field_type: Label,
    data_position: u64,
    data_size: u32,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct FieldHeader {
    field_type: Label,
    data_size: u16,
}

fn read_field<'a, R>(reader: &'a mut R) -> Result<(Field<'a, R>, u64)>
where R: Read + Seek {
    let header0: FieldHeader = reader.read_struct()?;
    let mut field_type = header0.field_type;
    let mut data_size = header0.data_size as u32;
    if field_type == Label(*b"XXXX") {
        data_size = reader.read_struct()?;
        let header1: FieldHeader = reader.read_struct()?;
        field_type = header1.field_type;
    };
    let data_position = reader.stream_position()?;
    Ok((Field {
        reader,
        field_type,
        data_position,
        data_size,
    }, data_position + data_size as u64))
}
