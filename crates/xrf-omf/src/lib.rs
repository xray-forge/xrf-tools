#![doc = include_str!("../README.md")]

pub(crate) mod omf_file;
pub(crate) mod omf_motions_processor;

pub use crate::omf_file::*;
pub use crate::omf_motions_processor::*;
