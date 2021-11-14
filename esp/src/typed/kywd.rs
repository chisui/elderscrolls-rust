use std::io::{Read, Seek};

use crate::raw;

use crate::typed::record::{FieldError, Record, RecordError, RecordType, unwarp_field};
use crate::typed::types::{Color, EditorID};


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct KYWD {
    pub key: EditorID,
    pub color: Option<Color>,
}
impl KYWD {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field,
            key: Option<EditorID>, value: Option<Color>) -> Result<(Option<EditorID>, Option<Color>), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => {
                if key.is_some() {
                    Err(FieldError::Duplicate)
                } else {
                    let data = reader.content(&field)?;
                    Ok((Some(data), value))
                }
            },
            Some("CNAM") => {
                let data = reader.content(&field)?;
                Ok((key, Some(data)))
            },
            _ => Err(FieldError::Unexpected)?,
        }
    }
}
impl Record for KYWD {
    fn record_type(&self) -> RecordType {
        RecordType::KYWD
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = (None, None);
        
        for field in reader.fields(&rec)? {
            tmp = KYWD::handle_field(reader, &field, tmp.0, tmp.1)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        let key = unwarp_field(tmp.0, b"EDID")?;
        Ok(Self { key, color: tmp.1 })
    }
}