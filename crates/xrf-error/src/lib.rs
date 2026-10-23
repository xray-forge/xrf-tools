#![doc = include_str!("../README.md")]

mod error;
mod from;
mod types;

pub use crate::error::XrfError;
pub use crate::types::XrfResult;
