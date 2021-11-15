use std::convert::TryFrom;
use std::io::{Read, Seek};

use crate::raw;

use crate::typed::record::{FieldError, Record, RecordError, RecordType, unwarp_field};
use crate::typed::types::{Color, EditorID};


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct KYWD {
    pub key: EditorID,
    pub color: Option<Color>,
}
impl TryFrom<PartKYWD> for KYWD {
    type Error = RecordError;
    fn try_from(tmp: PartKYWD) -> Result<Self, Self::Error> {
        Ok(Self {
            key: unwarp_field(tmp.edid, b"EDID")?,
            color: tmp.cnam,
        })
    }
}
#[derive(Debug, Clone, Default)]
struct PartKYWD {
    edid: Option<EditorID>,
    cnam: Option<Color>,
}
impl KYWD {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field, tmp: &mut PartKYWD) -> Result<(), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => tmp.edid = Some(reader.content(&field)?),
            Some("CNAM") => tmp.cnam = Some(reader.content(&field)?),
            _ => return Err(FieldError::Unexpected),
        }
        Ok(())
    }
}
impl Record for KYWD {
    fn record_type(&self) -> RecordType {
        RecordType::KYWD
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = PartKYWD::default();
        
        for field in reader.fields(&rec)? {
            Self::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }
        
        Self::try_from(tmp)
    }
}