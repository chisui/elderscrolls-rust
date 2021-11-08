use std::mem::size_of;
use std::io::{self, Read, Result, Seek};
use std::str::{self, FromStr};

use enumflags2::{BitFlags, bitflags};
use bytemuck::{Pod, Zeroable};

use crate::bin::{Readable, ReadableParam, VarSize, read_struct};


#[derive(Debug, Clone, Copy)]
pub enum EntryType {
    GRUP,
    Record(RecordType),
}
impl VarSize for EntryType {
    fn size(&self) -> usize { size_of::<[u8; 4]>() }
}
impl Readable for EntryType {
    fn read_bin<R: Read>(reader: R) -> Result<Self> {
        let bytes: [u8; 4] = read_struct(reader)?;
        let name =  str::from_utf8(&bytes)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        if name == "GRUP" {
            Ok(EntryType::GRUP)
        } else {
            RecordType::from_str(name)
                .map(EntryType::Record)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct RawGroupHeader {
    data_size: u32,
    label: [u8; 4],
    group_type: u32,
    timestamp: u16,
    version_control_info: VersionControlInfo,
    _unknown: u32,
}

#[derive(
    Debug, Clone, Copy,
    PartialEq, Eq, PartialOrd, Ord,
    strum::Display, strum::IntoStaticStr, strum::EnumString,
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
    #[strum(message = "[[Skyrim Mod:Actor Value Indices//Actor Value Codes|Actor Values]]/Perk Tree Graphics")]
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
    #[strum(message = "Dual Cast Data (''possibly unused'')")]
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

#[derive(Debug, Clone)]
pub struct GenericRecord {
    pub record_type: RecordType,
    pub flags: u32,
    pub id: u32,
    pub timestamp: u16,
    pub version_control_info: VersionControlInfo,
    pub internal_version: u16,
    pub fields: Vec<GenericField>,
}
impl VarSize for GenericRecord {
    fn size(&self) -> usize {
        EntryType::Record(self.record_type).size()
            + size_of::<RawRecordHeader>()
            + self.fields.size()
    }
}
impl ReadableParam<RecordType> for GenericRecord {
    fn read_with_param<R: Read>(mut reader: R, record_type: RecordType) -> Result<Self> {
        let header = RawRecordHeader::read_bin(&mut reader)?;
        
        let mut read_bytes = 0;
        let mut fields: Vec<GenericField> = Vec::new();
        while read_bytes < header.data_size as usize {
            let field_header = read_struct::<RawFieldHeader, _>(&mut reader)?;
            println!("{:#?}", field_header);
            // TODO XXXX field bullshit
            let field_type = str::from_utf8(&field_header.field_type)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
                .map(String::from)?;
            let data = u8::read_bin_many(&mut reader, field_header.field_size as usize)?;
    
            read_bytes += size_of::<RawFieldHeader>() + data.len();
            fields.push(GenericField {
                field_type,
                data,
            });
        }

        Ok(Self {
            record_type,
            flags: header.flags,
            id: header.id,
            timestamp: header.timestamp,
            version_control_info: header.version_control_info,
            internal_version: header.internal_version,
            fields,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct RawRecordHeader {
    data_size: u32,
    flags: u32,
    id: u32,
    timestamp: u16,
    version_control_info: VersionControlInfo,
    internal_version: u16,
    _unknown: u16,
}
derive_var_size_via_size_of!(RawRecordHeader);
derive_readable_via_pod!(RawRecordHeader);
derive_writable_via_pod!(RawRecordHeader);

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct VersionControlInfo {
    last_user: u8,
    current_user: u8,
}
derive_var_size_via_size_of!(VersionControlInfo);
derive_readable_via_pod!(VersionControlInfo);
derive_writable_via_pod!(VersionControlInfo);

pub enum RecordFlag {
    DeletedRecord = 0x00000020,
    Constant = 0x00000040,
    MustUpdateAnims = 0x00000100, 
    /// Quest item
    /// Persistent reference 
    PersistentRef = 0x00000400,
    InitiallyDisabled = 0x00000800,
    Ignored = 0x00001000,
    LOD = 0x00008000,
    Compressed = 0x00040000,
    IsMarker = 0x00800000,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TES4Flag {
    /// Master file
    ESM = 0x00000001,
    /// this will make Skyrim load the .STRINGS, .DLSTRINGS, and .ILSTRINGS files associated with the mod. If this flag is not set, lstrings are treated as zstrings.
    Localized = 0x00000080,
    /// Light Master File. See: https://www.creationkit.com/fallout4/index.php?title=Data_File
    ESL = 0x00000200,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum REFRFlag {
    /// Hidden From Local Map (Needs Confirmation: Related to shields)
    HiddenFromLocalMapShield = 0x00000040,
    /// (REFR) Hidden from local map / MotionBlurCastsShadows
    HiddenFromLocalMap = 0x00000200,    
    NoAiAcquire = 0x02000000,
    Inaccessible = 0x00000100,
    ReflectedByAutoWater = 0x10000000,
    NoHavokSettle = 0x20000000,
    NoRespawn = 0x40000000,
    MultiBound = 0x80000000,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ACHRFlag {
    StartsDead = 0x00000200,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LSCRFlag {
    DisplayInMainMenu = 0x00000400,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ACTIFlag {
    RandomAnimationStart = 0x00010000,
    /// Dangerous Can't be set without Ignore Object Interaction
    Dangerous = 0x00020000,
    /// Ignore Object Interaction. Sets Dangerous Automatically
    IgnoreObjectInteraction = 0x00100000,
    Obstacle = 0x02000000,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CELLFlag {
    /// Off limits Interior cell
    OffLimits = 0x00020000,
    CanNotWait = 0x00080000,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FURNFlag {
    MustExitToTalk = 0x10000000,
    ChildCanUse = 0x20000000,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IDLMFlag {
    ChildCanUse = 0x20000000,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NavMeshFlag {
    Filter = 0x04000000,
    BoundingBox = 0x08000000,
    Ground = 0x40000000,
}

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NoFlag {
    Dummy = 0x1,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct RawFieldHeader {
    pub field_type: [u8; 4],
    pub field_size: u16,
}

#[derive(Debug, Clone)]
pub struct GenericField {
    pub field_type: String,
    pub data: Vec<u8>,
}
impl VarSize for GenericField {
    fn size(&self) -> usize {
        size_of::<RawFieldHeader>()
            + self.data.len()
    }
}
impl Readable for GenericField {
    fn read_bin<R: Read>(mut reader: R) -> Result<Self> {
        let header = read_struct::<RawFieldHeader, _>(&mut reader)?;
        let field_type = str::from_utf8(&header.field_type)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
            .map(String::from)?;
        let data = u8::read_bin_many(&mut reader, header.field_size as usize)?;

        Ok(Self {
            field_type,
            data,
        })
    }
}
