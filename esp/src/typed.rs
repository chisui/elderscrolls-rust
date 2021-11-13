use std::convert::TryFrom;
use std::io::{self, Read, Seek};
use std::str::{self, FromStr, Utf8Error};
use std::fmt;

use strum::{self, EnumMessage};
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;
use bytemuck::{Pod, PodCastError, Zeroable};

use crate::raw;



#[derive(Debug, Error)]
pub enum EspError {
    #[error("{0}")] Record(#[from] RecordError),
    #[error("{} {1}", fmt_group_info(&.0))] Group(raw::GroupInfo, Box<EspError>),
    #[error("{0}")] IO(#[from] io::Error),
    #[error("Unexpected group type {0:?}")]
    GroupUnexpected(GroupType),
    #[error("Unknown group type {0}")]
    GroupUnknown(#[from] TryFromPrimitiveError<GroupType>),
    #[error("Only a single {0} group is permitted")]
    GroupDuplicate(GroupType),
    #[error("{0}")]
    BadGroupInfo(#[from] GroupInfoError),
    #[error("Record {0} missing")]
    RecordMissing(RecordType),
    #[error("Unexpected record {0}")]
    RecordUnexpected(raw::Label),
    #[error("Unknown record {0}")]
    RecordUnknown(raw::Label),
    #[error("Only a single {0} record is permitted")]
    RecordDuplicate(RecordType),
}
fn fmt_group_info(grp: &raw::GroupInfo) -> String {
    if let Ok(grpi) = GroupInfo::try_from(grp) {
        format!("{}", grpi)
    } else {
        format!("{}", grp)
    }
}
#[derive(Debug, Error)]
pub enum RecordError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("Unexpected field {0}")]
    UnexpectedField(raw::Label),
    #[error("Missing field {0}")]
    MissingField(raw::Label),
    #[error("Only a single {0} field is permitted")]
    DuplicateField(raw::Label),
    #[error("{0} {1}")] Field(raw::Label, FieldError),
}
#[derive(Debug, Error)]
pub enum FieldError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("{0}")] Cast(PodCastError),
    #[error("{0}")] Utf8(#[from] Utf8Error),
    #[error("Unexpected")] Unexpected,
    #[error("Unexpected field size {actual} (expected {expected})")]
    Unexpectedize {
        actual: usize,
        expected: usize,
    },
}

pub type EspRes<A> = Result<A, EspError>;

#[derive(Debug, Clone)]
pub struct EspFile<R> {
    reader: raw::EspReader<R>,
    tes4: TES4,
    groups: Vec<TopLevelGroup>,
}
impl<R> EspFile<R> {
    pub fn header(&self) -> &TES4 {
        &self.tes4
    }
    pub fn top_level_groups(&self) -> &Vec<TopLevelGroup> {
        &self.groups
    }
}

impl<R> EspFile<R>
where R: Read + Seek {
    pub fn read(r: R) -> EspRes<Self> {
        let mut reader = raw::EspReader::new(r);
        let mut tes4 = None;
        let mut groups = Vec::new();

        for entry in reader.top_level_entries()? {
            match entry {
                raw::Entry::Record(r) => {
                    if r.record_type == raw::Label(*b"TES4") {
                        if tes4 == None {
                            let rec = TES4::read_rec(&mut reader, &r)?;
                            tes4 = Some(rec);
                        } else {
                            return Err(EspError::RecordDuplicate(RecordType::TES4));
                        }
                    } else {
                        return Err(EspError::RecordUnexpected(r.record_type));
                    }
                },
                raw::Entry::Group(g) => {
                    let info = GroupInfo::try_from(&g.group_info)
                        .map_err(|err| EspError::BadGroupInfo(err))?;
                    if let GroupInfo::TopLevel(rec) = info {
                        let tlg = TopLevelGroup::read_group(&mut reader, rec, &g)?;
                        groups.push(tlg);
                    } else {
                        return Err(EspError::GroupUnexpected(info.group_type()));
                    }
                },
            }
        }

        Ok(Self {
            reader,
            tes4: tes4.ok_or(EspError::RecordMissing(RecordType::TES4))?,
            groups,
        })
    }
}

#[repr(u32)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive,
    strum::Display,
)]
pub enum GroupType {
    TopLevel = 0,
    WorldChildren = 1,
    CellBlockInterior = 2,
    CellSubBlockInterior = 3,
    CellBlockExterior = 4,
    CellSubBlockExterior = 5,
    CellChildren = 6,
    TopicChildren = 7,
    CellPersistentChilden = 8,
    CellTemporaryChildren = 9,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct FormId(u32);
impl fmt::Display for FormId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}
impl From<raw::Label> for FormId {
    fn from(l: raw::Label) -> Self {
        Self(u32::from_le_bytes(l.0))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct BlockId(u32);
impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}
impl From<raw::Label> for BlockId {
    fn from(l: raw::Label) -> Self {
        Self(u32::from_le_bytes(l.0))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point2D<C> {
    x: C,
    y: C,
}
impl From<raw::Label> for Point2D<u16> {
    fn from(l: raw::Label) -> Self {
        Self {
            x: u16::from_le_bytes([
                l.0[0],
                l.0[1],
            ]),
            y: u16::from_le_bytes([
                l.0[2],
                l.0[3],
            ]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupInfo {
    TopLevel(RecordType),
    World(FormId),
    CellBlock {
        block_type: CellBlockType,
        info: BlockInfo,
    },
    CellChildren {
        form: FormId,
        child_type: CellChildType,
    },
    Topic(FormId),
}
impl GroupInfo {
    pub fn group_type(&self) -> GroupType {
        match self {
            GroupInfo::TopLevel(_) => GroupType::TopLevel,
            GroupInfo::World(_) => GroupType::WorldChildren,
            GroupInfo::CellBlock { block_type, info } => match (block_type, info) {
                (CellBlockType::Block, BlockInfo::Interior(_)) => GroupType::CellBlockInterior,
                (CellBlockType::Block, BlockInfo::Exterior(_)) => GroupType::CellBlockExterior,
                (CellBlockType::SubBlock, BlockInfo::Interior(_)) => GroupType::CellSubBlockInterior,
                (CellBlockType::SubBlock, BlockInfo::Exterior(_)) => GroupType::CellSubBlockExterior,
            },
            GroupInfo::CellChildren { child_type, .. } => match child_type {
                CellChildType::Normal => GroupType::CellChildren,
                CellChildType::Persistent => GroupType::CellPersistentChilden,
                CellChildType::Temporary => GroupType::CellTemporaryChildren,
            },
            GroupInfo::Topic(_) => GroupType::TopicChildren,
        }
    }    
}
impl fmt::Display for GroupInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupInfo::TopLevel(r) => write!(f, "{}", r),
            GroupInfo::World(fid) => write!(f, "W{}", fid),
            GroupInfo::CellBlock {
                block_type,
                info
            } => write!(f, "{bt}-{info}",
                bt = match block_type {
                    CellBlockType::Block => "B",
                    CellBlockType::SubBlock => "b",
                },
                info = info),
            GroupInfo::CellChildren {
                form, 
                child_type 
            } => write!(f, "{}{}",
                form,
                match child_type {
                    CellChildType::Normal => "",
                    CellChildType::Persistent => "p",
                    CellChildType::Temporary => "t",
                }),
            GroupInfo::Topic(fid) => write!(f, "topic{}", fid),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellBlockType {
    Block,
    SubBlock,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockInfo {
    Interior(BlockId),
    Exterior(Point2D<u16>),
}
impl fmt::Display for BlockInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockInfo::Interior(b) => write!(f, "{}", b),
            BlockInfo::Exterior(Point2D { x, y }) => write!(f, "{},{}", x, y),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellChildType {
    Normal,
    Persistent,
    Temporary,
}
#[derive(Debug, Error)]
pub enum GroupInfoError {
    #[error("Unknown group type {0}")]
    UnknownType(#[from] TryFromPrimitiveError<GroupType>),
    #[error("Malformed group type {0}")]
    MalformedType(#[from] RecordTypeError),
    #[error("Unknown record type {0}")]
    UnknownRecordType(raw::Label),
}
impl TryFrom<&raw::GroupInfo> for GroupInfo {
    type Error = GroupInfoError;

    fn try_from(grp: &raw::GroupInfo) -> Result<Self, GroupInfoError> {
        Ok(match GroupType::try_from_primitive(grp.group_type)? {
            GroupType::TopLevel => {
                let record_type = RecordType::try_from(grp.label)?;
                GroupInfo::TopLevel(record_type)
            },
            GroupType::WorldChildren => {
                let form = FormId::from(grp.label);
                GroupInfo::World(form)
            },
            GroupType::CellBlockInterior => {
                let block = BlockId::from(grp.label);
                GroupInfo::CellBlock {
                    block_type: CellBlockType::Block,
                    info: BlockInfo::Interior(block),
                }
            },
            GroupType::CellSubBlockInterior => {
                let block = BlockId::from(grp.label);
                GroupInfo::CellBlock {
                    block_type: CellBlockType::SubBlock,
                    info: BlockInfo::Interior(block),
                }
            },
            GroupType::CellBlockExterior => {
                let point = Point2D::from(grp.label);
                GroupInfo::CellBlock {
                    block_type: CellBlockType::Block,
                    info: BlockInfo::Exterior(point),
                }
            },
            GroupType::CellSubBlockExterior => {
                let point = Point2D::from(grp.label);
                GroupInfo::CellBlock {
                    block_type: CellBlockType::SubBlock,
                    info: BlockInfo::Exterior(point),
                }
            },
            GroupType::CellChildren => {
                GroupInfo::CellChildren {
                    form: FormId::from(grp.label),
                    child_type: CellChildType::Normal,
                }
            },
            GroupType::TopicChildren => {
                GroupInfo::Topic(FormId::from(grp.label))
            },
            GroupType::CellPersistentChilden => {
                GroupInfo::CellChildren {
                    form: FormId::from(grp.label),
                    child_type: CellChildType::Persistent,
                }
            },
            GroupType::CellTemporaryChildren => {
                GroupInfo::CellChildren {
                    form: FormId::from(grp.label),
                    child_type: CellChildType::Temporary,
                }
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TopLevelGroup {
    pub record_type: RecordType,
}
impl TopLevelGroup {
    fn read_group<R: Read + Seek>(reader: &mut raw::EspReader<R>, record_type: RecordType, group: &raw::Group) -> EspRes<Self> {
        Ok(Self {
            record_type,
        })
    }
}
impl Group for TopLevelGroup {
    fn group_info(&self) -> GroupInfo {
        GroupInfo::TopLevel(self.record_type)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Zeroable, Pod)]
pub struct ObjectId(u32);

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

pub trait Record: Sized {
    fn record_type(&self) -> RecordType;

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: &raw::Record) -> Result<Self, RecordError>;
}
pub trait Group {
    fn group_info(&self) -> GroupInfo;
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TES4 {
    pub version: f32,
    pub len: u32,
    pub next_object_id: ObjectId,
    pub author: Option<String>,
    pub description: Option<String>,
    pub masters: Vec<MasterFile>,
    pub overridden_forms: Vec<FormId>,
    pub tagifiable_strings_len: u32,
    pub increment: Option<u32>,
}
impl TES4 {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field, res: (&mut TES4, Option<String>)) -> Result<Option<String>, FieldError> {
        #[repr(C)]
        #[derive(Debug, Clone, Copy, Zeroable, Pod)]
        struct HEDR {
            version: f32,
            len: u32,
            next_object_id: ObjectId,
        }
        let tes4 = res.0;

        match field.field_type.as_str() {
            Some("HEDR") => {
                let HEDR {
                    version,
                    len,
                    next_object_id,
                } = reader.cast_content(&field)?;
                tes4.version = version;
                tes4.len = len;
                tes4.next_object_id = next_object_id;
                Ok(None)
            },
            Some("CNAM") => {
                tes4.author = Some(zstring_content(reader, &field)?);
                Ok(None)
            },
            Some("SNAM") => {
                tes4.description = Some(zstring_content(reader, &field)?);
                Ok(None)
            },
            Some("MAST") => {
                let s = zstring_content(reader, &field)?;
                Ok(Some(s))
            },
            Some("DATA") => {
                if let Some(file) = &res.1 {
                    let size = reader.cast_content(&field)?;
                    tes4.masters.push(MasterFile {
                        file: file.to_owned(),
                        size,
                    });
                    Ok(None)
                } else {
                    Err(FieldError::Unexpected)
                }
            },
            Some("ONAM") => {
                tes4.overridden_forms = reader.cast_all_content(&field)?;
                Ok(None)
            },
            Some("INTV") => {
                tes4.tagifiable_strings_len = reader.cast_content(&field)?;
                Ok(None)
            },
            Some("INCC") => {
                tes4.increment = Some(reader.cast_content(&field)?);
                Ok(None)
            },
            _ => Err(FieldError::Unexpected),
        }
    }
}
impl Record for TES4 {
    fn record_type(&self) -> RecordType {
        RecordType::TES4
    }
    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: &raw::Record) -> Result<Self, RecordError> {
        let mut tes4 = TES4::default();
        let mut last_mast: Option<String> = None;

        for field in reader.fields(rec)? {
            last_mast = TES4::handle_field(reader, &field, (&mut tes4, last_mast))
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Ok(tes4)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GMST {
    pub key: String,
    pub value: TypedValue,
}
impl GMST {
    fn handle_field<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field,
            res: (Option<String>, Option<TypedValue>)) -> Result<(Option<String>, Option<TypedValue>), FieldError> {
        
        match field.field_type.as_str() {
            Some("EDID") => {
                let s = zstring_content(reader, &field)?;
                Ok((Some(s), res.1))
            },
            Some("DATA") => {
                let data = reader.content(&field)?;
                Ok((res.0, Some(TypedValue(data))))
            },
            _ => Err(FieldError::Unexpected)?,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypedValue(Vec<u8>);
impl Record for GMST {
    fn record_type(&self) -> RecordType {
        RecordType::GMST
    }

    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, rec: &raw::Record) -> Result<Self, RecordError> {
        let mut tmp = (None, None);
        
        for field in reader.fields(rec)? {
            tmp = GMST::handle_field(reader, &field, tmp)
                .map_err(|err| RecordError::Field(field.field_type, err))?;
        }

        Ok(Self {
            key: tmp.0.ok_or_else(|| RecordError::MissingField(raw::Label(*b"EDID")))?,
            value: tmp.1.ok_or_else(|| RecordError::MissingField(raw::Label(*b"DATA")))?,
        })
    }
}

#[derive(Debug, Error)]
pub enum StringError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("{0}")] Utf8(#[from] str::Utf8Error),
}
impl From<StringError> for FieldError {
    fn from(err: StringError) -> Self {
        match err {
            StringError::IO(e) => Self::from(e),
            StringError::Utf8(e) => Self::from(e),
        }
    }
}
fn zstring_content<R: Read + Seek>(reader: &mut raw::EspReader<R>, field: &raw::Field) -> Result<String, StringError> {
    let bytes = reader.content(&field)?;
    let s = str::from_utf8(&bytes[0 .. bytes.len() - 1])?;
    Ok(s.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MasterFile {
    pub file: String,
    pub size: u64,
}


#[cfg(test)]
pub(crate) mod test {
    use std::fs::File;
    use std::io::Result;

    use super::*;

    #[test]
    fn load_unoffical_patch() -> Result<()> {
        let f = File::open("../test-data/unofficialSkyrimSEpatch.esp")?;
        match EspFile::read(f) {
            Ok(f) => {
                println!("{:?}", f.header().author);
                for group in f.top_level_groups() {
                    println!("{}: {}", group.group_info(), group.record_type.description());
                }
                Ok(())
            },
            Err(e) => {
                println!("ERROR: {}", e);
                Err(io::Error::new(io::ErrorKind::Other, e))
            },
        }
    }
}
