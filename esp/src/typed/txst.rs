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
            id: unwarp_field(value.id, b"EDID")?,
            obnd: unwarp_field(value.obnd, b"OBND")?,
            color: unwarp_field(value.color, b"TX00")?,
            normal: value.normal,
            mask: value.mask,
            tone_or_glow: value.tone_or_glow,
            detail: value.detail,
            env: value.env,
            multilayer: value.multilayer,
            specular: value.specular,
            decal: value.decal,
            flags: unwarp_field(value.flags, b"DNAM")?,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PartTXST {
    pub id: Option<EditorID>,
    pub obnd: Option<[u8; 12]>,
    pub color: Option<Path>,
    pub normal: Option<Path>,
    pub mask: Option<Path>,
    pub tone_or_glow: Option<Path>,
    pub detail: Option<Path>,
    pub env: Option<Path>,
    pub multilayer: Option<Path>,
    pub specular: Option<Path>,
    pub decal: Option<DecalData>,
    pub flags: Option<BitFlags<TXSTFlags>>,
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
            Some("EDID") => {
                let data = reader.content(&field)?;
                tmp.id = Some(data);
            },
            Some("OBND") => {
                let obnd = reader.cast_content(&field)?;
                tmp.obnd = Some(obnd);
            },
            Some("TX00") => {
                let data = reader.content(&field)?;
                tmp.color = Some(data);
            },
            Some("TX01") => {
                let data = reader.content(&field)?;
                tmp.normal = Some(data);
            },
            Some("TX02") => {
                let data = reader.content(&field)?;
                tmp.mask = Some(data);
            },
            Some("TX03") => {
                let data = reader.content(&field)?;
                tmp.tone_or_glow = Some(data);
            },
            Some("TX04") => {
                let data = reader.content(&field)?;
                tmp.detail = Some(data);
            },
            Some("TX05") => {
                let data = reader.content(&field)?;
                tmp.env = Some(data);
            },
            Some("TX06") => {
                let data = reader.content(&field)?;
                tmp.multilayer = Some(data);
            },
            Some("TX07") => {
                let data = reader.content(&field)?;
                tmp.specular = Some(data);
            },
            Some("DODT") => {
                let raw_decl: RawDecalData = reader.cast_content(&field)?;
                tmp.decal = Some(raw_decl.into());
            },
            Some("DNAM") => {
                let flags: u16 = reader.cast_content(&field)?;
                tmp.flags = Some(BitFlags::from_bits_truncate(flags));
            },
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
            TXST::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Self::try_from(tmp)
    }
}