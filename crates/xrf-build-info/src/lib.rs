//! What a built binary can say about its own provenance.

mod build_info;
mod build_kind;
mod emit;

pub use crate::build_info::BuildInfo;
pub use crate::build_kind::BuildKind;
pub use crate::emit::emit;
