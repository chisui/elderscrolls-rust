use std::mem::size_of;
use std::io::{Read, Result};
use std::{fmt, str};

use bytemuck::{Pod, Zeroable};

use crate::bin::{Readable, read_struct, read_to_end, read_to_eof};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct Label([u8; 4]);
impl Label {
    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }
}
impl Readable for Label {
    fn read<R: Read>(reader: R) -> Result<(Self, usize)> {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entry {
    Record(Record),
    Group(Group),
}
impl Readable for Entry {
    fn read<R: Read>(mut reader: R) -> Result<(Self, usize)> {
        let (l, s) = read_struct(&mut reader)?;
        if l == Label(*b"GRUP") {
            Group::read(reader)
                .map(|(g, gs)| (Entry::Group(g), gs + s))
        } else {
            Record::read_after_label(reader, l)
                .map(|(r, rs)| (Entry::Record(r), rs + s))
        }
    }    
}

pub fn read<R: Read>(reader: R) -> Result<Vec<Entry>> {
    let (ex, _) = read_to_eof(reader)?;
    Ok(ex)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Record {
    pub record_type: Label,
    pub flags: u32,
    pub id: u32,
    pub timestamp: u16,
    pub version_control_info: VersionControlInfo,
    pub internal_version: u16,
    pub fields: Vec<Field>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct RecordHeader {
    data_size: u32,
    flags: u32,
    id: u32,
    timestamp: u16,
    version_control_info: VersionControlInfo,
    internal_version: u16,
    _unknown: u16,
}
impl Record {
   fn read_after_label<R: Read>(mut reader: R, record_type: Label) -> Result<(Self, usize)> {
        let (RecordHeader {
            data_size,
            flags,
            id,
            timestamp,
            version_control_info,
            internal_version,
            ..
        }, header_size) = read_struct(&mut reader)?;

        let (fields, fields_size) = read_to_end(&mut reader, data_size as usize)?;

        Ok((Record {
            record_type,
            flags,
            id,
            timestamp,
            version_control_info,
            internal_version,
            fields,
        }, header_size + fields_size))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group {
    pub label: Label,
    pub group_type: u32,
    pub timestamp: u16,
    pub version_control_info: VersionControlInfo,
    pub members: Vec<Entry>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct GroupHeader {
    data_size: u32,
    label: Label,
    group_type: u32,
    timestamp: u16,
    version_control_info: VersionControlInfo,
    _unknown: u32,
}
const TOTAL_GROUP_HEADER_SIZE: usize
    = size_of::<[u8; 4]>()
    + size_of::<GroupHeader>();
impl Readable for Group {
    fn read<R: Read>(mut reader: R) -> Result<(Self, usize)> {
        let (GroupHeader {
            data_size,
            label,
            group_type,
            timestamp,
            version_control_info,
            ..
        }, header_size) = read_struct(&mut reader)?;

        let (members, members_size) = read_to_end(&mut reader, data_size as usize - TOTAL_GROUP_HEADER_SIZE)?;

        Ok((Group {
            label,
            group_type,
            timestamp,
            version_control_info,
            members,
        }, header_size + members_size))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field {
    field_type: Label,
    data: Vec<u8>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct FieldHeader {
    field_type: Label,
    field_size: u16,
}
impl Readable for Field {
    fn read<R: Read>(mut reader: R) -> Result<(Self, usize)> {
        let (f, mut header_size) = read_struct::<FieldHeader, _>(&mut reader)?;
        let (field_type, data_size) = if f.field_type == Label(*b"XXXX") {
            assert_eq!(f.field_size, 4, "Fields with label \"XXXX\" have to have a size of 4, but got {}", f.field_size);
            let (size, _) = read_struct::<u32, _>(&mut reader)?;
            let (g, second_header_size) = read_struct::<FieldHeader, _>(&mut reader)?;
            header_size += second_header_size;
            (g.field_type, size as usize)
        } else {
            (f.field_type, f.field_size as usize)
        };

        let mut data = vec![0u8; data_size as usize];
        reader.read_exact(&mut data)?;
        Ok((Field {
            field_type,
            data,
        }, header_size + data_size as usize))
    }
}
