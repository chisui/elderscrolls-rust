use std::mem::size_of;
use std::io::{self, Read, Seek, SeekFrom, Result};
use std::{fmt, str};

use thiserror::Error;
use bytemuck::{Pod, Zeroable};

use crate::bin::ReadStructExt;


#[derive(Debug, Clone, Copy)]
pub struct EspReader<R> {
    reader: R,
}
impl<R> EspReader<R>
where R: Read + Seek {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn top_level_entries(&mut self) -> Result<Vec<Entry>> {
        let end = self.reader.stream_len()?;
        self.read_entries(0, end)
    }

    pub fn entries(&mut self, group: &Group) -> Result<Vec<Entry>> {
        self.read_entries(group.position + TOTAL_GROUP_HEADER_SIZE as u64, group.data_size as u64)
    }

    fn read_entries(&mut self, start: u64, size: u64) -> Result<Vec<Entry>> {
        let end = start + size;
        let mut current = self.reader.seek(SeekFrom::Start(start))?;
        let mut entries = Vec::new();
        while current < end {
            let entry = self.read_entry()?;
            current = self.reader.seek(SeekFrom::Current(entry.data_size() as i64))?;
            entries.push(entry);
        }
        Ok(entries)
    }

    fn read_entry(&mut self) -> Result<Entry> {
        let l= self.reader.read_struct()?;
        if l == Label(*b"GRUP") {
            self.read_group()
                .map(Entry::Group)
        } else {
            self.read_record_after_label(l)
                .map(Entry::Record)
        }
    }

    fn read_group(&mut self) -> Result<Group>
    where R: Read + Seek {
        let position = self.reader.stream_position()? - size_of::<Label>() as u64;
        let GroupHeader {
            data_size,
            group_info,
            timestamp,
            version_control_info,
            ..
        } = self.reader.read_struct()?;
        Ok(Group {
            data_size: data_size - TOTAL_GROUP_HEADER_SIZE as u32,
            position,

            group_info,
            timestamp,
            version_control_info,
        })
    }

    fn read_record_after_label(&mut self, record_type: Label) -> Result<Record> {
        let position = self.reader.stream_position()? - size_of::<Label>() as u64;
        let RecordHeader {
            data_size,
            flags,
            id,
            timestamp,
            version_control_info,
            internal_version,
            ..
        } = self.reader.read_struct()?;
        
        Ok(Record {
            data_size,
            position,

            record_type,
            flags,
            id,
            timestamp,
            version_control_info,
            internal_version,
        })
    }

    pub fn fields(&mut self, rec: &Record) -> Result<Vec<Field>> {
        let end = rec.position + TOTAL_RECORD_HEADER_SIZE as u64 + rec.data_size as u64;
        let mut current = self.reader.seek(SeekFrom::Start(rec.position + TOTAL_RECORD_HEADER_SIZE as u64))?;
        let mut fields = Vec::new();
        while current < end {
            let field = self.read_field()?;
            current = self.reader.seek(SeekFrom::Current(field.data_size as i64))?;
            fields.push(field);
        }
        Ok(fields)
    }

    fn read_field(&mut self) -> Result<Field> {
        let header0: FieldHeader = self.reader.read_struct()?;
        let mut field_type = header0.field_type;
        let mut data_size = header0.data_size as u32;
        if field_type == Label(*b"XXXX") {
            data_size = self.reader.read_struct()?;
            let header1: FieldHeader = self.reader.read_struct()?;
            field_type = header1.field_type;
        };
        let data_position = self.reader.stream_position()?;
        Ok(Field {
            field_type,
            data_position,
            data_size,
        })
    }

    pub fn content(&mut self, field: &Field) -> Result<Vec<u8>> {
        self.reader.seek(SeekFrom::Start(field.data_position))?;
        let mut data = vec![0u8; field.data_size as usize];
        self.reader.read_exact(&mut data)?;
        Ok(data)
    }

    pub fn cast_content<P: Pod>(&mut self, field: &Field) -> Result<P> {
        if size_of::<P>() != field.data_size as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidData, 
                CastError::Size(size_of::<P>(), field.data_size as usize)))
        }
        self.reader.seek(SeekFrom::Start(field.data_position))?;
        self.reader.read_struct()
    }

    pub fn cast_all_content<P: Pod>(&mut self, field: &Field) -> Result<Vec<P>> {
        if field.data_size as usize % size_of::<P>() != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, 
                CastError::Size(size_of::<P>(), field.data_size as usize)))
        }
        self.reader.seek(SeekFrom::Start(field.data_position))?;
        let mut vec = Vec::new();
        for _ in 0 .. (field.data_size as usize / size_of::<P>()) {
            vec.push(self.reader.read_struct()?);
        }
        Ok(vec)
    }
}

#[derive(Debug, Error)]
pub enum CastError {
    #[error("Expected {0} bytes but got {1}")]
    Size(usize, usize),
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct Label(pub [u8; 4]);
impl Label {
    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }
}
impl AsRef<[u8; 4]> for Label {
    fn as_ref(&self) -> &[u8; 4] {
        &self.0
    }
}
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(s) => f.write_str(s),
            None => write!(f, "0x{:x}", u32::from_le_bytes(self.0)),
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
pub enum Entry {
    Record(Record),
    Group(Group),
}
impl Entry {
    fn data_size(&self) -> u64 {
        match self {
            Entry::Record(r) => r.data_size as u64,
            Entry::Group(g) => g.data_size as u64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Record {
    position: u64,
    data_size: u32,
    pub record_type: Label,
    pub flags: u32,
    pub id: u32,
    pub timestamp: Timestamp,
    pub version_control_info: VersionControlInfo,
    pub internal_version: u16,
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
const TOTAL_RECORD_HEADER_SIZE: usize
    = size_of::<Label>()
    + size_of::<RecordHeader>();

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group {
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
impl fmt::Display for GroupInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}${:x}", self.group_type, u32::from_le_bytes(self.label.0))
    }
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field {
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


#[cfg(test)]
pub(crate) mod test {
    use std::fs::File;
    use std::io::Result;

    use super::*;

    #[test]
    fn load_unoffical_patch() -> Result<()> {
        let f = File::open("../test-data/unofficialSkyrimSEpatch.esp")?;
        let mut reader = EspReader::new(f);

        let entries = reader.top_level_entries()?;

        for entry in entries {
            match entry {
                Entry::Record(r) => {
                    println!("Record: {}", r.record_type);
                    let fields = reader.fields(&r)?;
                    for field in fields {
                        println!("  {}", field.field_type);
                    }
                },
                Entry::Group(g) => {
                    println!("Group: {}", g.group_info.label);
                }
            }
        }
        Ok(())
    }
}
