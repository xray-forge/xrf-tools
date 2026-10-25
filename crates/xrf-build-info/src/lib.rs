#![doc = include_str!("../README.md")]

mod build_info;
mod build_info_emitter;
mod build_kind;

pub use crate::build_info::BuildInfo;
pub use crate::build_info_emitter::BuildInfoEmitter;
pub use crate::build_kind::BuildKind;
