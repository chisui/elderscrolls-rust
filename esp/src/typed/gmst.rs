use std::io::{Read, Seek};

use crate::raw;

use crate::typed::record::{FieldError, Record, RecordError, RecordType};
use crate::typed::types::{EditorID, TypedValue};

use super::record::unwarp_field;


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct GMST {
    pub key: EditorID,
    pub value: TypedValue,
}
impl GMST {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field,
            key: Option<EditorID>, value: Option<Vec<u8>>) -> Result<(Option<EditorID>, Option<Vec<u8>>), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => {
                if key.is_some() {
                    Err(FieldError::Duplicate)
                } else {
                    let data = reader.content(&field)?;
                    Ok((Some(data), value))
                }
            },
            Some("DATA") => {
                let data = reader.content(&field)?;
                Ok((key, Some(data)))
            },
            _ => Err(FieldError::Unexpected)?,
        }
    }
}
impl Record for GMST {
    fn record_type(&self) -> RecordType {
        RecordType::GMST
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = (None, None);
        
        for field in reader.fields(&rec)? {
            tmp = GMST::handle_field(reader, &field, tmp.0, tmp.1)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        let key = unwarp_field(tmp.0, b"EDID")?;
        let c = key.0.chars().next().ok_or_else(|| RecordError::Field(raw::Label(*b"EDID"), FieldError::Unexpectedize{
            actual: 0,
            expected: 1,
        }))?;
        let data = unwarp_field(tmp.1, b"DATA")?;
        let value = TypedValue::new(c, data)
            .map_err(|err| RecordError::Field(raw::Label(*b"DATA"), FieldError::Cast(err)))?;

        Ok(Self { key, value })
    }
}
