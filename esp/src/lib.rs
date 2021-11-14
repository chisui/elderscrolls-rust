#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization, seek_stream_len, buf_read_has_data_left)]
#[macro_use]
mod bin;
pub mod raw;
mod typed;

pub use crate::typed::*;
