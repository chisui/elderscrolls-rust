use std::convert::TryFrom;
use std::io::Read;
use std::io::Seek;

use bytemuck::Pod;
use bytemuck::Zeroable;
use enumflags2::{bitflags, BitFlags};

use crate::raw;

use super::record::{FieldError, Record, RecordError, RecordType, unwarp_field};
use super::types::{Color, EditorID, Path};


#[derive(Debug, Clone, PartialEq)]
pub struct TXST {
    pub id: EditorID,
    pub obnd: [u8; 12],
    pub color: Path,
    pub normal: Option<Path>,
    pub mask: Option<Path>,
    pub tone_or_glow: Option<Path>,
    pub detail: Option<Path>,
    pub env: Option<Path>,
    pub multilayer: Option<Path>,
    pub specular: Option<Path>,
    pub decal: Option<DecalData>,
    pub flags: BitFlags<TXSTFlags>,
}
impl TryFrom<PartTXST> for TXST {
    type Error = RecordError;
    fn try_from(value: PartTXST) -> Result<Self, Self::Error> {
        Ok(Self {
            id: unwarp_field(value.edid, b"EDID")?,
            obnd: unwarp_field(value.obnd, b"OBND")?,
            color: unwarp_field(value.tx00, b"TX00")?,
            normal: value.tx01,
            mask: value.tx02,
            tone_or_glow: value.tx03,
            detail: value.tx04,
            env: value.tx05,
            multilayer: value.tx06,
            specular: value.tx07,
            decal: value.dodt.map(DecalData::from),
            flags: unwarp_field(value.dnam, b"DNAM")?,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
struct PartTXST {
    edid: Option<EditorID>,
    obnd: Option<[u8; 12]>,
    tx00: Option<Path>,
    tx01: Option<Path>,
    tx02: Option<Path>,
    tx03: Option<Path>,
    tx04: Option<Path>,
    tx05: Option<Path>,
    tx06: Option<Path>,
    tx07: Option<Path>,
    dodt: Option<RawDecalData>,
    dnam: Option<BitFlags<TXSTFlags>>,
}
#[bitflags]
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TXSTFlags {
    NoSpecularMap = 0x01,
    FacegenTextures = 0x02,
    ModelSpaceNormalMap = 0x04,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecalData {
    pub width_min: f32,
    pub width_max: f32,
    pub height_min: f32,
    pub height_max: f32,
    pub shininess: f32,
    pub parallax_scale: f32,
    pub parallax_passes: u8,
    pub flags: BitFlags<DecalFlags>,
    pub color: Color,
}
impl From<RawDecalData> for DecalData {
    fn from(raw: RawDecalData) -> Self {
        Self {
            width_min: raw.width_min,
            width_max: raw.width_max,
            height_min: raw.height_min,
            height_max: raw.height_max,
            shininess: raw.shininess,
            parallax_scale: raw.parallax_scale,
            parallax_passes: raw.parallax_passes,
            flags: BitFlags::from_bits_truncate(raw.flags),
            color: raw.color,
        }
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Zeroable, Pod)]
struct RawDecalData {
    width_min: f32,
    width_max: f32,
    height_min: f32,
    height_max: f32,
    shininess: f32,
    parallax_scale: f32,
    parallax_passes: u8,
    flags: u8,
    _unknown: u16,
    color: Color,
}
#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DecalFlags {
    Parallax = 0x01,
    AlphaBlending = 0x02,
    AlphaTesting = 0x04,
    NotForSubTextures = 0x08,
}
impl TXST {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field,
            tmp: &mut PartTXST) -> Result<(), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => tmp.edid = Some(reader.content(&field)?),
            Some("OBND") => tmp.obnd = Some(reader.cast_content(&field)?),
            Some("TX00") => tmp.tx00 = Some(reader.content(&field)?),
            Some("TX01") => tmp.tx01 = Some(reader.content(&field)?),
            Some("TX02") => tmp.tx02 = Some(reader.content(&field)?),
            Some("TX03") => tmp.tx03 = Some(reader.content(&field)?),
            Some("TX04") => tmp.tx04 = Some(reader.content(&field)?),
            Some("TX05") => tmp.tx05 = Some(reader.content(&field)?),
            Some("TX06") => tmp.tx06 = Some(reader.content(&field)?),
            Some("TX07") => tmp.tx07 = Some(reader.content(&field)?),
            Some("DODT") => tmp.dodt = Some(reader.cast_content(&field)?),
            Some("DNAM") => tmp.dnam = Some(reader.content(&field)?),
            _ => return Err(FieldError::Unexpected),
        }
        Ok(())
    }
}
impl Record for TXST {
    fn record_type(&self) -> RecordType {
        RecordType::TXST
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = PartTXST::default();
        
        for field in reader.fields(&rec)? {
            Self::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Self::try_from(tmp)
    }
}