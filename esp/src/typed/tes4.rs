use std::io::{Read, Seek};

use bytemuck::{Pod, Zeroable};

use crate::raw;
use crate::typed::types::ZString;
use crate::typed::types::{FormId, ObjectId};
use crate::typed::record::{FieldError, Record, RecordError, RecordType};


#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TES4 {
    pub version: f32,
    pub len: u32,
    pub next_object_id: ObjectId,
    pub author: Option<String>,
    pub description: Option<String>,
    pub masters: Vec<MasterFile>,
    pub overridden_forms: Vec<FormId>,
    pub tagifiable_strings_len: u32,
    pub increment: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MasterFile {
    pub file: String,
    pub size: u64,
}
impl TES4 {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field, res: (&mut TES4, Option<String>)) -> Result<Option<String>, FieldError> {
        #[repr(C)]
        #[derive(Debug, Clone, Copy, Zeroable, Pod)]
        struct HEDR {
            version: f32,
            len: u32,
            next_object_id: ObjectId,
        }
        let tes4 = res.0;

        match field.field_type.as_str() {
            Some("HEDR") => {
                let HEDR {
                    version,
                    len,
                    next_object_id,
                } = reader.cast_content(&field)?;
                tes4.version = version;
                tes4.len = len;
                tes4.next_object_id = next_object_id;
                Ok(None)
            },
            Some("CNAM") => {
                let ZString(s) = reader.content(&field)?;
                tes4.author = Some(s);
                Ok(None)
            },
            Some("SNAM") => {
                let ZString(s) = reader.content(&field)?;
                tes4.description = Some(s);
                Ok(None)
            },
            Some("MAST") => {
                let ZString(s) = reader.content(&field)?;
                Ok(Some(s))
            },
            Some("DATA") => {
                if let Some(file) = &res.1 {
                    let size = reader.content(&field)?;
                    tes4.masters.push(MasterFile {
                        file: file.to_owned(),
                        size,
                    });
                    Ok(None)
                } else {
                    Err(FieldError::Unexpected)
                }
            },
            Some("ONAM") => {
                tes4.overridden_forms = reader.content(&field)?;
                Ok(None)
            },
            Some("INTV") => {
                tes4.tagifiable_strings_len = reader.cast_content(&field)?;
                Ok(None)
            },
            Some("INCC") => {
                tes4.increment = Some(reader.cast_content(&field)?);
                Ok(None)
            },
            _ => Err(FieldError::Unexpected),
        }
    }
}
impl Record for TES4 {
    fn record_type(&self) -> RecordType {
        RecordType::TES4
    }
    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tes4 = TES4::default();
        let mut last_mast: Option<String> = None;

        for field in reader.fields(&rec)? {
            last_mast = TES4::handle_field(reader, &field, (&mut tes4, last_mast))
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Ok(tes4)
    }
}
