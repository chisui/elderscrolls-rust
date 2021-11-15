use std::convert::TryFrom;
use std::io::{Read, Seek};

use num_enum::TryFromPrimitive;

use crate::bin::{ReadStructExt, Readable};
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
impl TryFrom<PartGLOB> for GLOB {
    type Error = RecordError;
    fn try_from(tmp: PartGLOB) -> Result<Self, Self::Error> {
        Ok(Self {
            id: unwarp_field(tmp.edid, b"EDID")?,
            value: VarValue::new(
                unwarp_field(tmp.fnam, b"FNAM")?,
                unwarp_field(tmp.fltv, b"FLTV")?,
            ),
            bounds: tmp.obnd,
            constant: false,
        })
    }
}
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GLOBFlag {
    Const = 0x40,
}
#[derive(Debug, Clone, Default)]
struct PartGLOB {
    edid: Option<EditorID>,
    fnam: Option<VarType>,
    fltv: Option<f32>,
    obnd: Option<ObjectBounds>,
    // script: Option<ScriptInfo>,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
enum VarType {
    Short = b's',
    Long  = b'l',
    Float = b'f',
}
impl<R: Read> Readable<R> for VarType {
    type Error = FieldError;
    fn read_val(reader: &mut R) -> Result<Self, Self::Error> {
        let data: u8 = reader.read_struct()?;
        let res = VarType::try_from_primitive(data)?;
        Ok(res)
    }
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
            Some("EDID") => tmp.edid = Some(reader.content(&field)?),
            Some("FNAM") => tmp.fnam = Some(reader.content(&field)?),
            Some("FLTV") => tmp.fltv = Some(reader.cast_content(&field)?),
            Some("OBND") => tmp.obnd = Some(reader.content(&field)?),
            _ => return Err(FieldError::Unexpected),
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
            Self::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        let mut res = Self::try_from(tmp)?;
        res.constant = rec.flags & GLOBFlag::Const as u32 == GLOBFlag::Const as u32;
        Ok(res)
    }
}
