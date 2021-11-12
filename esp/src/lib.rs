#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization, seek_stream_len)]
#[macro_use]
mod bin;
pub mod raw;
mod typed;

pub use crate::typed::*;
