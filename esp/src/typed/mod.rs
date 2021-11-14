use std::convert::TryFrom;
use std::io::{Read, Seek};

use crate::raw;

mod error;
mod types;
mod group;
mod record;
mod tes4;
mod gmst;
mod glob;
mod txst;
mod kywd;
use crate::typed::error::*;
use crate::typed::group::*;
use crate::typed::record::Record;
use crate::typed::tes4::TES4;
use crate::typed::gmst::GMST;
use crate::typed::glob::GLOB;
use crate::typed::txst::TXST;
use crate::typed::kywd::KYWD;

use self::record::{RecordError, RecordType};


pub fn read_esp<R>(r: R) -> EspRes<Vec<Entry>>
where R: Read + Seek {
    let mut reader = raw::EspReader::new(r);
    let mut entries = Vec::new();

    for raw_entry in reader.top_level_entries()? {
        let entry = Entry::read_entry(&mut reader, raw_entry)?;
        entries.push(entry);
    }

    Ok(entries)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Record(SomeRecord),
    Group(Group),
}
impl Entry {
    fn read_entry<R>(reader: &mut raw::EspReader<R>, entry: raw::Entry) -> EspRes<Self>
    where R: Read + Seek {
        match entry {
            raw::Entry::Record(rec) => {
                let rec = SomeRecord::read_rec(reader, rec)?;
                Ok(Entry::Record(rec))
            },
            raw::Entry::Group(group) => Group::read_group(reader, &group)
                .map(Entry::Group),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub info: GroupInfo,
    pub entries: Vec<Entry>
}
impl Group {
    fn read_group<R: Read + Seek>(reader: &mut raw::EspReader<R>, group: &raw::Group) -> EspRes<Self> {
        let info = GroupInfo::try_from(&group.group_info)?;
        let mut entries = Vec::new();

        for raw_entry in reader.entries(group)? {
            let entry = Entry::read_entry(reader, raw_entry)
                .map_err(|err| EspError::Group(info.clone(), Box::new(err)))?;
            entries.push(entry)
        }
        Ok(Self { info, entries })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SomeRecord {
    TES4(TES4),
    GMST(GMST),
    KYWD(KYWD),
    TXST(TXST),
    GLOB(GLOB),
    Other(RecordType, raw::Record),
}
impl Record for SomeRecord {
    fn record_type(&self) -> RecordType {
        match self {
            SomeRecord::TES4(_) => RecordType::TES4,
            SomeRecord::GMST(_) => RecordType::GMST,
            SomeRecord::KYWD(_) => RecordType::KYWD,
            SomeRecord::TXST(_) => RecordType::TXST,
            SomeRecord::GLOB(_) => RecordType::GLOB,
            SomeRecord::Other(t, _) => *t,
        }
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        match RecordType::try_from(rec.record_type)? {
            RecordType::TES4 => TES4::read_rec(reader, rec)
                .map(SomeRecord::TES4),
            RecordType::GMST => GMST::read_rec(reader, rec)
                .map(SomeRecord::GMST),
            RecordType::KYWD => KYWD::read_rec(reader, rec)
                .map(SomeRecord::KYWD),
            RecordType::TXST => TXST::read_rec(reader, rec)
                .map(SomeRecord::TXST),
            RecordType::GLOB => GLOB::read_rec(reader, rec)
                .map(SomeRecord::GLOB),
            t => Ok(SomeRecord::Other(t, rec)),
        }
    }
}
