use std::io;

use num_enum::TryFromPrimitiveError;
use thiserror::Error;

use crate::raw;

use crate::typed::group::{GroupInfo, GroupInfoError, GroupType};
use crate::typed::record::{RecordType, RecordError};


#[derive(Debug, Error)]
pub enum EspError {
    #[error("{0}")] Record(#[from] RecordError),
    #[error("{0} {1}")] Group(GroupInfo, Box<EspError>),
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
    #[error("Only a single {0} record is permitted")]
    RecordDuplicate(RecordType),
}

pub type EspRes<A> = Result<A, EspError>;
