use std::convert::TryFrom;
use std::io::{Read, Seek};

use bytemuck::{Pod, Zeroable};

use crate::raw;

use crate::typed::types::*;
use crate::typed::record::*;


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct CLAS {
    pub id: EditorID,
    pub name: String,
    pub description: String,
    pub menu_image: Option<Path>,
    pub skill_training: ActorValue,
    pub skill_training_level: u8,
    pub skill_weights: SkillWeights,
    pub bleedout_default: f32,
    pub voice_points: u32,
    pub health_weight: u8,
    pub magicka_weight: u8,
    pub stamina_weight: u8,
    pub flags: u8,
}
impl TryFrom<PartCLAS> for CLAS {
    type Error = RecordError;
    fn try_from(tmp: PartCLAS) -> Result<Self, Self::Error> {
        let data = unwarp_field(tmp.data, b"DATA")?;
        Ok(Self {
            id: unwarp_field(tmp.edid, b"EDID")?,
            name: unwarp_field(tmp.full, b"FULL")?.to_string(),
            description: unwarp_field(tmp.desc, b"DESC")?.to_string(),
            menu_image: tmp.icon,
            skill_training: data.skill_training,
            skill_training_level: data.skill_training_level,
            skill_weights: data.skill_weights,
            bleedout_default: data.bleedout_default,
            voice_points: data.voice_points,
            health_weight: data.health_weight,
            magicka_weight: data.magicka_weight,
            stamina_weight: data.stamina_weight,
            flags: data.flags,
        })
    }
}
#[derive(Debug, Clone, Default)]
struct PartCLAS {
    edid: Option<EditorID>,
    full: Option<ZString>,
    desc: Option<ZString>,
    icon: Option<Path>,
    data: Option<CLASData>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Zeroable, Pod)]
struct CLASData {
    _unknown: u32,
    skill_training: ActorValue,
    skill_training_level: u8,
    skill_weights: SkillWeights,
    bleedout_default: f32,
    voice_points: u32,
    health_weight: u8,
    magicka_weight: u8,
    stamina_weight: u8,
    flags: u8,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct SkillWeights {
    pub one_handed: u8,
    pub two_handed: u8,
    pub marksman: u8,
    pub block: u8,
    pub smithing: u8,
    pub heavy_armor: u8,
    pub light_armor: u8,
    pub pickpocket: u8,
    pub lockpicking: u8,
    pub sneak: u8,
    pub alchemy: u8,
    pub speechcraft: u8,
    pub alteration: u8,
    pub conjuration: u8,
    pub destruction: u8,
    pub illusion: u8,
    pub restoration: u8,
    pub enchanting: u8,
}
impl CLAS {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field,
            tmp: &mut PartCLAS) -> Result<(), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => tmp.edid = Some(reader.content(&field)?),
            Some("FULL") => tmp.full = Some(reader.content(&field)?),
            Some("DESC") => tmp.desc = Some(reader.content(&field)?),
            Some("ICON") => tmp.icon = Some(reader.content(&field)?),
            Some("DATA") => tmp.data = Some(reader.cast_content(&field)?),
            _ => return Err(FieldError::Unexpected)?,
        }
        Ok(())
    }
}
impl Record for CLAS {
    fn record_type(&self) -> RecordType {
        RecordType::CLAS
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = PartCLAS::default();
        
        for field in reader.fields(&rec)? {
            Self::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Self::try_from(tmp)
    }
}
