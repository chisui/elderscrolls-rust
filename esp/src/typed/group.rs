use std::{convert::TryFrom, fmt};

use thiserror::Error;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};

use crate::raw;

use crate::typed::types::{BlockId, FormId, Point2D};
use crate::typed::record::{RecordType, RecordTypeError};


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
