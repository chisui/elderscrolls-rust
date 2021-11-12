use std::convert::TryFrom;
use std::io::{self, Read, Seek};
use std::str::{self, FromStr, Utf8Error};
use std::fmt;

use strum;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;
use bytemuck::{Pod, PodCastError, Zeroable};

use crate::raw;


#[derive(Debug, Clone, Default)]
pub struct EntryPath(pub Vec<raw::GroupInfo>);
impl EntryPath {
    fn append(&self, a: raw::GroupInfo) -> Self {
        let mut p = self.0.clone();
        p.push(a);
        Self(p)
    }
    fn record(&self, a: raw::Label) -> RecordPath {
        RecordPath(self.clone(), a)
    }
}
impl fmt::Display for EntryPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for grp in self.0.iter() {
            write!(f, "/")?;
            if let Ok(grpi) = GroupInfo::try_from(grp) {
                grpi.fmt(f)?;
            } else {
                grp.fmt(f)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RecordPath(EntryPath, raw::Label);
impl RecordPath {
    fn field(&self, f: raw::Label) -> FieldPath {
        FieldPath(self.clone(), f)
    }
}
impl fmt::Display for RecordPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

#[derive(Debug, Clone)]
pub struct FieldPath(pub RecordPath, pub raw::Label);
impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

#[derive(Debug, Error)]
pub enum EspError {
    #[error("{0}: {1}")] Group(EntryPath, GroupError),
    #[error("{0}: {1}")] Record(RecordPath, RecordError),
    #[error("{0}: {1}")] Field(FieldPath, FieldError),
}
#[derive(Debug, Error)]
pub enum GroupError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("Unexpected group type {0:?}")]
    GroupUnexpected(GroupType),
    #[error("Unknown group type {0}")]
    GroupUnknown(#[from] TryFromPrimitiveError<GroupType>),
    #[error("Only a single {0} group is permitted")]
    GroupDuplicate(GroupType),
    #[error("{0}")]
    BadGroupInfo(GroupInfoError),
    #[error("Record {0} missing")]
    RecordMissing(RecordType),
    #[error("Unexpected record {0}")]
    RecordUnexpected(raw::Label),
    #[error("Unknown record {0}")]
    RecordUnknown(raw::Label),
    #[error("Only a single {0} record is permitted")]
    RecordDuplicate(RecordType),
}
#[derive(Debug, Error)]
pub enum RecordError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("Unexpected field {0}")]
    UnexpectedField(raw::Label),
    #[error("Only a single {0} field is permitted")]
    DuplicateField(raw::Label),
}
#[derive(Debug, Error)]
pub enum FieldError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("{0}")] Cast(PodCastError),
    #[error("{0}")] Utf8(#[from] Utf8Error),
    #[error("Unexpected field size {1} (expected {0})")]
    UnexpectedFieldSize(usize, usize),
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

        let entry_path = EntryPath::default();
        for entry in reader.top_level_entries()
                .map_err(|err| EspError::Group(entry_path.clone(), err.into()))? {
            match entry {
                raw::Entry::Record(r) => {
                    if r.record_type == raw::Label(*b"TES4") {
                        if tes4 == None {
                            let rec = TES4::read_rec(&mut reader, &entry_path, &r)?;
                            tes4 = Some(rec);
                        } else {
                            return Err(EspError::Group(entry_path.clone(), GroupError::RecordDuplicate(RecordType::TES4)));
                        }
                    } else {
                        return Err(EspError::Group(entry_path.clone(), GroupError::RecordUnexpected(r.record_type)));
                    }
                },
                raw::Entry::Group(g) => {
                    let t = GroupType::try_from_primitive(g.group_info.group_type)
                        .map_err(|err| EspError::Group(entry_path.clone(), GroupError::GroupUnknown(err)))?;
                    if t == GroupType::TopLevel {
                        let tlg = TopLevelGroup::read_group(&mut reader, &g)?;
                        groups.push(tlg);
                    } else {
                        return Err(EspError::Group(entry_path.clone(), GroupError::GroupUnexpected(t)));
                    }
                },
            }
        }

        Ok(Self {
            reader,
            tes4: tes4.ok_or(EspError::Group(entry_path, GroupError::RecordMissing(RecordType::TES4)))?,
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
    fn read_group<R: Read + Seek>(reader: &mut raw::EspReader<R>, group: &raw::Group) -> EspRes<Self> {
        Ok(Self {
            record_type: RecordType::TES4,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Zeroable, Pod)]
pub struct ObjectId(u32);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
    strum::IntoStaticStr, strum::EnumString, strum::Display
)]
pub enum RecordType {
    TES4,
}

#[derive(Debug, Error)]
pub enum RecordTypeError {
    #[error("Record type is not a string: {0}")]
    NotString(raw::Label),
    #[error("Can't parse record type {0}")]
    Malformed(#[from] strum::ParseError),
}
impl TryFrom<raw::Label> for RecordType {
    type Error = RecordTypeError;

    fn try_from(l: raw::Label) -> Result<Self, Self::Error> {
        if let Some(s) = l.as_str() {
            let r = RecordType::from_str(s)?;
            Ok(r)
        } else {
            Err(RecordTypeError::NotString(l))
        }
    }
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


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct HEDR {
    version: f32,
    len: u32,
    next_object_id: ObjectId,
}
impl TES4 {
    fn read_rec<R: Read + Seek>(reader: &mut raw::EspReader<R>, entry_path: &EntryPath, rec: &raw::Record) -> EspRes<Self> {
        let mut tes4 = TES4::default();
        let mut last_mast: Option<String> = None;
        let rec_path = entry_path.record(rec.record_type);

        for field in reader.fields(rec)
                .map_err(|err| EspError::Record(rec_path.clone(), err.into()))? {
            let field_path = rec_path.field(field.field_type);
            let to_err = |err: io::Error| EspError::Field(field_path.clone(), err.into());
            match field.field_type.as_str() {
                Some("HEDR") => {
                    let HEDR {
                        version,
                        len,
                        next_object_id,
                    } = reader.cast_content(&field)
                            .map_err(to_err)?;
                    tes4.version = version;
                    tes4.len = len;
                    tes4.next_object_id = next_object_id;
                    last_mast = None;
                },
                Some("CNAM") => {
                    tes4.author = Some(zstring_content(reader, &field_path, &field)?);
                    last_mast = None;
                },
                Some("SNAM") => {
                    tes4.description = Some(zstring_content(reader, &field_path, &field)?);
                    last_mast = None;
                },
                Some("MAST") => {
                    last_mast = Some(zstring_content(reader, &field_path, &field)?);
                },
                Some("DATA") => {
                    if let Some(file) = last_mast {
                        let size = reader.cast_content(&field)
                            .map_err(to_err)?;
                        tes4.masters.push(MasterFile {
                            file,
                            size,
                        });
                    } else {
                        return Err(EspError::Record(rec_path, RecordError::UnexpectedField(field.field_type)));
                    }
                    last_mast = None;
                },
                Some("ONAM") => {
                    tes4.overridden_forms = reader.cast_all_content(&field)
                        .map_err(to_err)?;
                    last_mast = None;
                },
                Some("INTV") => {
                    tes4.tagifiable_strings_len = reader.cast_content(&field)
                        .map_err(to_err)?;
                    last_mast = None;
                },
                Some("INCC") => {
                    tes4.increment = reader.cast_content(&field)
                        .map_err(to_err)
                        .map(Option::Some)?;
                    last_mast = None;
                },
                _ => return Err(EspError::Record(rec_path, RecordError::UnexpectedField(field.field_type))),
            }
        }

        Ok(tes4)
    }
}

fn zstring_content<R: Read + Seek>(reader: &mut raw::EspReader<R>, path: &FieldPath, field: &raw::Field) -> EspRes<String> {
    let bytes = reader.content(&field)
        .map_err(|err| EspError::Field(path.clone(), err.into()))?;
    let s = str::from_utf8(&bytes[0 .. bytes.len() - 1])
        .map_err(|err| EspError::Field(path.clone(), err.into()))?;
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
                println!("{:#?}", f.header());
                Ok(())
            },
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
}
