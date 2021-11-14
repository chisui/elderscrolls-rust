use std::io::{Read, Seek};

use num_enum::TryFromPrimitive;

use crate::raw;

use crate::typed::types::*;
use crate::typed::record::*;


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct GLOB {
    pub id: EditorID,
    pub constant: bool,
    pub value: VarValue,
    pub bounds: Option<ObjectBounds>,
    // pub script: Option<ScriptInfo>,
}
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GLOBFlag {
    Const = 0x40,
}
#[derive(Debug, Clone, Default)]
struct PartGLOB {
    id: Option<EditorID>,
    var_type: Option<VarType>,
    value: Option<f32>,
    bounds: Option<ObjectBounds>,
    // script: Option<ScriptInfo>,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
enum VarType {
    Short = b's',
    Long  = b'l',
    Float = b'f',
}
#[derive(Debug, Clone, PartialEq, PartialOrd,)]
pub enum VarValue {
    Short(u16),
    Long(i32),
    Float(f32),
}
impl VarValue {
    fn new(t: VarType, value: f32) -> Self {
        match t {
            VarType::Short => VarValue::Short(value as u16),
            VarType::Long  => VarValue::Long(value as i32),
            VarType::Float => VarValue::Float(value),
        }
    }
}
impl GLOB {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field,
            tmp: &mut PartGLOB) -> Result<(), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => {
                let data = reader.content(&field)?;
                tmp.id = Some(data);
            },
            Some("FNAM") => {
                let data: u8 = reader.cast_content(&field)?;
                let t = VarType::try_from_primitive(data)?;
                tmp.var_type = Some(t);
            },
            Some("FLTV") => {
                let data = reader.cast_content(&field)?;
                tmp.value = Some(data);
            },
            Some("OBND") => {
                // let data = reader.cast_content(&field)?;
                // tmp.bounds = Some(data);
            },
            _ => return Err(FieldError::Unexpected)?,
        }
        Ok(())
    }
}
impl Record for GLOB {
    fn record_type(&self) -> RecordType {
        RecordType::GLOB
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = PartGLOB::default();
        
        for field in reader.fields(&rec)? {
            GLOB::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Ok(Self {
            id: unwarp_field(tmp.id, b"EDID")?,
            value: VarValue::new(
                unwarp_field(tmp.var_type, b"FNAM")?,
                unwarp_field(tmp.value, b"FLTV")?,
            ),
            bounds: tmp.bounds,
            constant: rec.flags & GLOBFlag::Const as u32 == GLOBFlag::Const as u32
        })
    }
}
