use std::convert::TryFrom;
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
impl TryFrom<PartGMST> for GMST {
    type Error = RecordError;
    fn try_from(tmp: PartGMST) -> Result<Self, Self::Error> {
        let key = unwarp_field(tmp.edid, b"EDID")?;
        let c = key.0.chars().next().ok_or_else(|| RecordError::Field(raw::Label(*b"EDID"), FieldError::Unexpectedize{
            actual: 0,
            expected: 1,
        }))?;
        let data = unwarp_field(tmp.data, b"DATA")?;
        let value = TypedValue::new(c, data)
            .map_err(|err| RecordError::Field(raw::Label(*b"DATA"), FieldError::Cast(err)))?;

        Ok(Self { key, value })
    }
}
#[derive(Debug, Clone, Default)]
struct PartGMST {
    edid: Option<EditorID>,
    data: Option<Vec<u8>>,
}
impl GMST {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field, tmp: &mut PartGMST) -> Result<(), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => tmp.edid = Some(reader.content(&field)?),
            Some("DATA") => tmp.data = Some(reader.content(&field)?),
            _ => return Err(FieldError::Unexpected),
        }
        Ok(())
    }
}
impl Record for GMST {
    fn record_type(&self) -> RecordType {
        RecordType::GMST
    }
    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = PartGMST::default();
        
        for field in reader.fields(&rec)? {
            Self::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Self::try_from(tmp)
    }
}
