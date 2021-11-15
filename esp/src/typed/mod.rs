use std::convert::TryFrom;
use std::io::{Read, Seek};

use crate::raw;

mod error;
mod types;
mod group;
mod record;
mod clas;
mod fact;
mod glob;
mod gmst;
mod kywd;
mod tes4;
mod txst;
use crate::typed::error::*;
use crate::typed::group::*;
use crate::typed::record::Record;
use crate::typed::clas::CLAS;
use crate::typed::fact::FACT;
use crate::typed::glob::GLOB;
use crate::typed::gmst::GMST;
use crate::typed::kywd::KYWD;
use crate::typed::tes4::TES4;
use crate::typed::txst::TXST;

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
    CLAS(CLAS),
    FACT(FACT),
    GLOB(GLOB),
    GMST(GMST),
    KYWD(KYWD),
    TES4(TES4),
    TXST(TXST),
    Other(RecordType, raw::Record),
}
impl Record for SomeRecord {
    fn record_type(&self) -> RecordType {
        match self {
            Self::CLAS(_) => RecordType::CLAS,
            Self::FACT(_) => RecordType::FACT,
            Self::GLOB(_) => RecordType::GLOB,
            Self::GMST(_) => RecordType::GMST,
            Self::KYWD(_) => RecordType::KYWD,
            Self::TES4(_) => RecordType::TES4,
            Self::TXST(_) => RecordType::TXST,
            SomeRecord::Other(t, _) => *t,
        }
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        match RecordType::try_from(rec.record_type)? {
            RecordType::CLAS => CLAS::read_rec(reader, rec)
                .map(Self::CLAS),
            RecordType::FACT => FACT::read_rec(reader, rec)
                .map(Self::FACT),
            RecordType::GLOB => GLOB::read_rec(reader, rec)
                .map(Self::GLOB),
            RecordType::GMST => GMST::read_rec(reader, rec)
                .map(Self::GMST),
            RecordType::KYWD => KYWD::read_rec(reader, rec)
                .map(Self::KYWD),
            RecordType::TES4 => TES4::read_rec(reader, rec)
                .map(Self::TES4),
            RecordType::TXST => TXST::read_rec(reader, rec)
                .map(Self::TXST),
            t => Ok(SomeRecord::Other(t, rec)),
        }
    }
}
