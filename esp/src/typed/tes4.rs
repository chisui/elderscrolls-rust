use std::convert::TryFrom;
use std::io::{Read, Seek};

use bytemuck::{Pod, Zeroable};

use crate::raw;
use crate::typed::types::ZString;
use crate::typed::types::{FormId, ObjectId};
use crate::typed::record::{FieldError, Record, RecordError, RecordType};

use super::record::unwarp_field;


#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TES4 {
    pub version: f32,
    pub top_level_groups_len: u32,
    pub next_object_id: ObjectId,
    pub author: Option<String>,
    pub description: Option<String>,
    pub masters: Vec<MasterFile>,
    pub overridden_forms: Vec<FormId>,
    pub tagifiable_strings_len: u32,
    pub increment: Option<u32>,
}
impl TryFrom<PartTES4> for TES4 {
    type Error = RecordError;
    fn try_from(tmp: PartTES4) -> Result<Self, Self::Error> {
        let hedr = unwarp_field(tmp.hedr, b"HEDR")?;
        Ok(Self {
            version: hedr.version,
            top_level_groups_len: hedr.len,
            next_object_id: hedr.next_object_id,
            author: tmp.cnam.map(String::from),
            description: tmp.snam.map(String::from),
            masters: tmp.mast.iter()
                .map(MasterFile::try_from)
                .collect::<Result<Vec<MasterFile>, RecordError>>()?,
            overridden_forms: tmp.onam.unwrap_or_default(),
            tagifiable_strings_len: unwarp_field(tmp.intv, b"INTV")?,
            increment: tmp.incc,
        })
    }
}
#[derive(Debug, Clone, Default)]
struct PartTES4 {
    hedr: Option<HEDR>,
    cnam: Option<ZString>,
    snam: Option<ZString>,
    mast: Vec<PartMasterFile>,
    onam: Option<Vec<FormId>>,
    intv: Option<u32>,
    incc: Option<u32>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct HEDR {
    version: f32,
    len: u32,
    next_object_id: ObjectId,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MasterFile {
    pub file: String,
    pub size: u64,
}
impl TryFrom<&PartMasterFile> for MasterFile {
    type Error = RecordError;
    fn try_from(tmp: &PartMasterFile) -> Result<Self, Self::Error> {
        Ok(Self {
            file: tmp.mast.to_string(),
            size: unwarp_field(tmp.data, b"DATA")?,
        })
    }
}
#[derive(Debug, Clone)]
struct PartMasterFile {
    mast: ZString,
    data: Option<u64>,
}
impl TES4 {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field, tmp: &mut PartTES4) -> Result<(), FieldError> {
        match field.field_type.as_str() {
            Some("HEDR") => tmp.hedr = Some(reader.cast_content(&field)?),
            Some("CNAM") => tmp.cnam = Some(reader.content(&field)?),
            Some("SNAM") => tmp.snam = Some(reader.content(&field)?),
            Some("MAST") => tmp.mast.push(PartMasterFile{ data: None, mast: reader.content(&field)? }),
            Some("DATA") => {
                let mast = tmp.mast.last_mut()
                    .ok_or(FieldError::Unexpected)?;
                if mast.data == None {
                    mast.data = Some(reader.content(&field)?);
                } else {
                    return Err(FieldError::Unexpected);
                }
            },
            Some("ONAM") => tmp.onam = Some(reader.content(&field)?),
            Some("INTV") => tmp.intv = Some(reader.cast_content(&field)?),
            Some("INCC") => tmp.incc = Some(reader.cast_content(&field)?),
            _ => return Err(FieldError::Unexpected),
        }
        Ok(())
    }
}
impl Record for TES4 {
    fn record_type(&self) -> RecordType {
        RecordType::TES4
    }
    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = PartTES4::default();

        for field in reader.fields(&rec)? {
            Self::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Self::try_from(tmp)
    }
}
