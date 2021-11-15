use std::convert::TryFrom;
use std::io::{self, BufRead, Read, Seek};

use bytemuck::{Pod, Zeroable};
use enumflags2::{bitflags, BitFlags};
use num_enum::TryFromPrimitive;

use crate::bin::{ReadStructExt, Readable};
use crate::raw;

use crate::typed::types::*;
use crate::typed::record::*;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FACT {
    pub id: EditorID,
    pub name: Option<LString>,
    pub relations: Vec<Relation>,
    pub flags: BitFlags<FACTFlag>,
    pub ranks: Vec<Rank>,
    pub jail_cell: Option<FormId>,
    pub wait_pos: Option<FormId>,
    pub stolen_goods_chest: Option<FormId>,
    pub player_chest: Option<FormId>,
    pub crime_factions: Option<FormId>,
    pub jail_outfit: Option<FormId>,
}
impl TryFrom<PartFACT> for FACT {
    type Error = RecordError;
    fn try_from(part: PartFACT) -> Result<Self, Self::Error> {
        Ok(FACT {
            id: unwarp_field(part.edid, b"EDID")?,
            name: part.full,
            relations: part.xnam,
            flags: unwarp_field(part.data, b"DATA")?,
            ranks: part.ranks,
            jail_cell: part.jail,
            wait_pos: part.wait,
            stolen_goods_chest: part.stol,
            player_chest: part.plcn,
            crime_factions: part.crgr,
            jail_outfit: part.jout,
        })
    }
}
#[bitflags]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive)]
pub enum FACTFlag {
    HiddenfromPC = 0x1,
    SpecialCombat = 0x2,
    TrackCrime = 0x40,
    IgnoreMurder = 0x80,
    IgnoreAssault = 0x100,
    IgnoreStealing = 0x200,
    IgnoreTrespass = 0x400,
    Donotreportcrimesagainstmembers = 0x800,
    CrimeGold,UseDefaults = 0x1000,
    IgnorePickpocket = 0x2000,
    Vendor = 0x4000,
    CanBeOwner = 0x8000,
    IgnoreWerewolf = 0x10000,
}
#[derive(Debug, Clone, Default)]
struct PartFACT {
    edid: Option<EditorID>,
    full: Option<LString>,
    xnam: Vec<Relation>,
    data: Option<BitFlags<FACTFlag>>,
    jail: Option<FormId>,
    wait: Option<FormId>,
    stol: Option<FormId>,
    plcn: Option<FormId>,
    crgr: Option<FormId>,
    jout: Option<FormId>,
    crva: Option<CrimeGold>,
    ranks: Vec<Rank>,
    vend: Option<FormId>,
    venc: Option<FormId>,
    venv: Option<VENV>,
    plvd: Option<PLVD>,
    // ctda: Option<CTDA>, TODO CTDA
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Relation {
    pub faction: FormId,
    pub unknown: u32,
    pub behavior: Behavior,
}
impl<R: Read> Readable<R> for Relation {
    type Error = FieldError;
    fn read_val(reader: &mut R) -> Result<Self, Self::Error> {
        let faction = reader.read_struct()?;
        let unknown = reader.read_struct()?;
        let behavior = reader.read_struct()?;
        Ok(Self {
            faction,
            unknown,
            behavior: Behavior::try_from_primitive(behavior)?,
        })
    }
}
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive)]
pub enum Behavior {
    Neutral = 0,
    Enemy = 1,
    Ally = 2,
    Friend = 3,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Zeroable, Pod)]
struct CrimeGoldHeader {
    arrest: u8,
    attack_on_sight: u8,
    murder: u16,
    assault: u16,
    trespass: u16,
    pickpocket: u16,
    unknwon: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CrimeGold {
    pub arrest: u8,
    pub attack_on_sight: u8,
    pub murder: u16,
    pub assault: u16,
    pub trespass: u16,
    pub pickpocket: u16,
    pub unknwon: u16,
    pub steal_mult: Option<f32>,
    pub escape: Option<u16>,
    pub werewolf: Option<u16>,
}
impl<R: BufRead> Readable<R> for CrimeGold {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        let header: CrimeGoldHeader = reader.read_struct()?;
        let steal_mult = if reader.has_data_left()? {
            Some(reader.read_struct()?)
        } else {
            None
        };
        let (escape, werewolf) = if reader.has_data_left()? {
            (
                Some(reader.read_struct()?),
                Some(reader.read_struct()?),
            )
        } else {
            (None, None)
        };
        Ok(Self {
            arrest: header.arrest,
            attack_on_sight: header.attack_on_sight,
            murder: header.murder,
            assault: header.assault,
            trespass: header.trespass,
            pickpocket: header.pickpocket,
            unknwon: header.unknwon,
            steal_mult,
            escape,
            werewolf,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank {
    pub id: u32,
    pub title_female: Option<LString>,
    pub title_male: Option<LString>,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Zeroable, Pod)]
struct VENV {
    hour_start: u16,
    hour_end: u16,
    radius: u32,
    fence: u8,
    complement: u8,
    unknown: u16,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Zeroable, Pod)]
struct PLVD {
    spec_type: u32,
    form: FormId,
    unknown: u32,
}
impl FACT {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field,
            tmp: &mut PartFACT) -> Result<(), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => tmp.edid = Some(reader.content(&field)?),
            Some("FULL") => tmp.full = Some(reader.content(&field)?),
            Some("XNAM") => tmp.xnam.push(reader.content(&field)?),
            Some("DATA") => tmp.data = Some(reader.content(&field)?),
            Some("JAIL") => tmp.jail = Some(reader.content(&field)?),
            Some("wait") => tmp.wait = Some(reader.content(&field)?),
            Some("STOL") => tmp.stol = Some(reader.content(&field)?),
            Some("PLCN") => tmp.plcn = Some(reader.content(&field)?),
            Some("CRGR") => tmp.crgr = Some(reader.content(&field)?),
            Some("JOUT") => tmp.jout = Some(reader.content(&field)?),
            Some("CRVA") => tmp.crva = Some(reader.content(&field)?),
            Some("RNAM") => {
                tmp.ranks.push(Rank {
                    id: reader.content(&field)?,
                    title_female: None,
                    title_male: None,
                })
            },
            Some("MNAM") => {
                let mut rank = tmp.ranks.last_mut()
                    .ok_or_else(|| FieldError::Unexpected)?;
                rank.title_male = Some(reader.content(&field)?);
            },
            Some("FNAM") => {
                let mut rank = tmp.ranks.last_mut()
                    .ok_or_else(|| FieldError::Unexpected)?;
                rank.title_female = Some(reader.content(&field)?);
            },
            Some("VEND") => tmp.vend = Some(reader.content(&field)?),
            Some("VENC") => tmp.venc = Some(reader.content(&field)?),
            Some("VENV") => tmp.venv = Some(reader.cast_content(&field)?),
            Some("PLVD") => tmp.plvd = Some(reader.cast_content(&field)?),
            Some("WAIT") => tmp.wait = Some(reader.content(&field)?),
            Some("CITC") => {}, // tmp.ctda = Some(reader.content(&field)?), TODO CTDA
            Some("CTDA") => {}, // tmp.ctda = Some(reader.content(&field)?), TODO CTDA
            _ => return Err(FieldError::Unexpected)?,
        }
        Ok(())
    }
}
impl Record for FACT {
    fn record_type(&self) -> RecordType {
        RecordType::FACT
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError> {
        let mut tmp = PartFACT::default();
        
        for field in reader.fields(&rec)? {
            Self::handle_field(reader, &field, &mut tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Self::try_from(tmp)
    }
}
