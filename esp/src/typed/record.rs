use std::convert::TryFrom;
use std::io::{self, Read, Seek};
use std::str::{self, FromStr};

use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use strum::{self, EnumMessage};
use bytemuck::PodCastError;
use thiserror::Error;

use crate::raw;
use crate::typed::types::StringError;


pub trait Record: Sized {
    fn record_type(&self) -> RecordType;

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: raw::Record) -> Result<Self, RecordError>;
}

pub fn unwarp_field<A>(opt: Option<A>, l: &[u8; 4]) -> Result<A, RecordError> {
    opt.ok_or(RecordError::MissingField(raw::Label(*l)))
}

#[derive(Debug, Error)]
pub enum RecordError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("{0}")] BadRecordType(#[from] RecordTypeError),
    #[error("Unexpected field {0}")]
    UnexpectedField(raw::Label),
    #[error("Missing field {0}")]
    MissingField(raw::Label),
    #[error("{0} {1}")] Field(raw::Label, FieldError),
}
#[derive(Debug, Error)]
pub enum FieldError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("{0}")] Cast(PodCastError),
    #[error("Illegal enum value {0}")] Enum(String),
    #[error("{0}")] Utf8(#[from] str::Utf8Error),
    #[error("Unexpected")] Unexpected,
    #[error("Duplicate")] Duplicate,
    #[error("Unexpected field size {actual} (expected {expected})")]
    Unexpectedize {
        actual: usize,
        expected: usize,
    },
}
impl From<StringError> for FieldError {
    fn from(err: StringError) -> Self {
        match err {
            StringError::IO(e) => Self::from(e),
            StringError::Utf8(e) => Self::from(e),
        }
    }
}
impl<A> From<TryFromPrimitiveError<A>> for FieldError
where
    A: TryFromPrimitive,
    <A as TryFromPrimitive>::Primitive: ToString,
{
    fn from(err: TryFromPrimitiveError<A>) -> Self {
        Self::Enum(err.number.to_string())
    }
}

impl RecordType {
    pub fn description(&self) -> &'static str {
        self.get_message().unwrap()
    }
}

#[derive(Debug, Error)]
pub enum RecordTypeError {
    #[error("Record type is not a string: {0}")]
    NotString(raw::Label),
    #[error("Can't parse record type {0}")]
    Malformed(raw::Label, strum::ParseError),
}
impl TryFrom<raw::Label> for RecordType {
    type Error = RecordTypeError;

    fn try_from(l: raw::Label) -> Result<Self, Self::Error> {
        if let Some(s) = l.as_str() {
            RecordType::from_str(s)
                .map_err(|err| RecordTypeError::Malformed(l, err))
        } else {
            Err(RecordTypeError::NotString(l))
        }
    }
}


#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
    strum::IntoStaticStr, strum::EnumString, strum::Display, strum::EnumMessage
)]
pub enum RecordType {
    #[strum(message = "Action")]
    AACT,
    #[strum(message = "Actor Reference")]
    ACHR,
    #[strum(message = "Activator")]
    ACTI,
    #[strum(message = "Addon Node")]
    ADDN,
    #[strum(message = "Potion")]
    ALCH,
    #[strum(message = "Ammo")]
    AMMO,
    #[strum(message = "Animation Object")]
    ANIO,
    #[strum(message = "Apparatus ''(probably unused)''")]
    APPA,
    #[strum(message = "Armor Addon (Model)")]
    ARMA,
    #[strum(message = "Armor")]
    ARMO,
    #[strum(message = "Art Object")]
    ARTO,
    #[strum(message = "Acoustic Space")]
    ASPC,
    #[strum(message = "Association Type")]
    ASTP,
    #[strum(message = "Actor Values/Perk Tree Graphics")]
    AVIF,
    #[strum(message = "Book")]
    BOOK,
    #[strum(message = "Body Part Data")]
    BPTD,
    #[strum(message = "Camera Shot")]
    CAMS,
    #[strum(message = "Cell")]
    CELL,
    #[strum(message = "Class")]
    CLAS,
    #[strum(message = "Color")]
    CLFM,
    #[strum(message = "Climate")]
    CLMT,
    #[strum(message = "Constructible Object (recipes)")]
    COBJ,
    #[strum(message = "Collision Layer")]
    COLL,
    #[strum(message = "Container")]
    CONT,
    #[strum(message = "Camera Path")]
    CPTH,
    #[strum(message = "Combat Style")]
    CSTY,
    #[strum(message = "Debris")]
    DEBR,
    #[strum(message = "Dialog Topic")]
    DIAL,
    #[strum(message = "Dialog Branch")]
    DLBR,
    #[strum(message = "Dialog View")]
    DLVW,
    #[strum(message = "Default Object Manager")]
    DOBJ,
    #[strum(message = "Door")]
    DOOR,
    #[strum(message = "Dual Cast Data (possibly unused)")]
    DUAL,
    #[strum(message = "Encounter Zone")]
    ECZN,
    #[strum(message = "Effect Shader")]
    EFSH,
    #[strum(message = "Enchantment")]
    ENCH,
    #[strum(message = "Equip Slot (flag-type values)")]
    EQUP,
    #[strum(message = "Explosion")]
    EXPL,
    #[strum(message = "Eyes")]
    EYES,
    #[strum(message = "Faction")]
    FACT,
    #[strum(message = "Flora")]
    FLOR,
    #[strum(message = "Form List (non-leveled list)")]
    FLST,
    #[strum(message = "Footstep")]
    FSTP,
    #[strum(message = "Footstep Set")]
    FSTS,
    #[strum(message = "Furniture")]
    FURN,
    #[strum(message = "Global Variable")]
    GLOB,
    #[strum(message = "Game Setting")]
    GMST,
    #[strum(message = "Grass")]
    GRAS,
    #[strum(message = "Form Group")]
    GRUP,
    #[strum(message = "Hazard")]
    HAZD,
    #[strum(message = "Head Part")]
    HDPT,
    #[strum(message = "Idle Animation")]
    IDLE,
    #[strum(message = "Idle Marker")]
    IDLM,
    #[strum(message = "Image Space Modifier")]
    IMAD,
    #[strum(message = "Image Space")]
    IMGS,
    #[strum(message = "Dialog Topic Info")]
    INFO,
    #[strum(message = "Ingredient")]
    INGR,
    #[strum(message = "Impact Data")]
    IPCT,
    #[strum(message = "Impact Data Set")]
    IPDS,
    #[strum(message = "Key")]
    KEYM,
    #[strum(message = "Keyword")]
    KYWD,
    #[strum(message = "Landscape")]
    LAND,
    #[strum(message = "Location Reference Type")]
    LCRT,
    #[strum(message = "Location")]
    LCTN,
    #[strum(message = "Lighting Template")]
    LGTM,
    #[strum(message = "Light")]
    LIGH,
    #[strum(message = "Load Screen")]
    LSCR,
    #[strum(message = "Land Texture")]
    LTEX,
    #[strum(message = "Leveled Item")]
    LVLI,
    #[strum(message = "Leveled Actor")]
    LVLN,
    #[strum(message = "Leveled Spell")]
    LVSP,
    #[strum(message = "Material Object")]
    MATO,
    #[strum(message = "Material Type")]
    MATT,
    #[strum(message = "Message")]
    MESG,
    #[strum(message = "Magic Effect")]
    MGEF,
    #[strum(message = "Misc. Object")]
    MISC,
    #[strum(message = "Movement Type")]
    MOVT,
    #[strum(message = "Movable Static")]
    MSTT,
    #[strum(message = "Music Type")]
    MUSC,
    #[strum(message = "Music Track")]
    MUST,
    #[strum(message = "Navigation (master data)")]
    NAVI,
    #[strum(message = "NavMesh")]
    NAVM,
    #[strum(message = "Note")]
    NOTE,
    #[strum(message = "Actor (NPC, Creature)")]
    NPC_,
    #[strum(message = "Outfit")]
    OTFT,
    #[strum(message = "AI Package")]
    PACK,
    #[strum(message = "Perk")]
    PERK,
    #[strum(message = "Placed grenade")]
    PGRE,
    #[strum(message = "Placed hazard")]
    PHZD,
    #[strum(message = "Projectile")]
    PROJ,
    #[strum(message = "Quest")]
    QUST,
    #[strum(message = "Race / Creature type")]
    RACE,
    #[strum(message = "Object Reference")]
    REFR,
    #[strum(message = "Region (Audio/Weather)")]
    REGN,
    #[strum(message = "Relationship")]
    RELA,
    #[strum(message = "Reverb Parameters")]
    REVB,
    #[strum(message = "Visual Effect")]
    RFCT,
    #[strum(message = "Scene")]
    SCEN,
    #[strum(message = "Scroll")]
    SCRL,
    #[strum(message = "Shout")]
    SHOU,
    #[strum(message = "Soul Gem")]
    SLGM,
    #[strum(message = "Story Manager Branch Node")]
    SMBN,
    #[strum(message = "Story Manager Event Node")]
    SMEN,
    #[strum(message = "Story Manager Quest Node")]
    SMQN,
    #[strum(message = "Sound Category")]
    SNCT,
    #[strum(message = "Sound Reference")]
    SNDR,
    #[strum(message = "Sound Output Model")]
    SOPM,
    #[strum(message = "Sound")]
    SOUN,
    #[strum(message = "Spell")]
    SPEL,
    #[strum(message = "Shader Particle Geometry")]
    SPGD,
    #[strum(message = "Static")]
    STAT,
    #[strum(message = "Talking Activator")]
    TACT,
    #[strum(message = "Plugin info / Header")]
    TES4,
    #[strum(message = "Tree")]
    TREE,
    #[strum(message = "Texture Set")]
    TXST,
    #[strum(message = "Voice Type")]
    VTYP,
    #[strum(message = "Water Type")]
    WATR,
    #[strum(message = "Weapon")]
    WEAP,
    #[strum(message = "Word Of Power")]
    WOOP,
    #[strum(message = "Worldspace")]
    WRLD,
    #[strum(message = "Weather")]
    WTHR,
}
