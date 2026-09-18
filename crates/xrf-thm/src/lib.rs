//! `.thm` texture descriptors: what the engine loads a texture as, and the bump pair beside it.

pub(crate) mod chunks;
pub mod thm_bump_mode;
pub mod thm_bump_patch_report;
pub mod thm_bump_processor;
pub(crate) mod thm_detail_usage;
pub(crate) mod thm_file;
pub(crate) mod thm_format;
pub(crate) mod thm_material;
pub(crate) mod thm_mip_filter;
pub(crate) mod thm_texture_flag;
pub(crate) mod thm_texture_flags;
pub(crate) mod thm_texture_type;

#[cfg(test)]
mod tests;

pub use crate::chunks::thm_bump_chunk::*;
pub use crate::chunks::thm_detail_chunk::*;
pub use crate::chunks::thm_extra_chunk::*;
pub use crate::chunks::thm_material_chunk::*;
pub use crate::chunks::thm_texture_param_chunk::*;
pub use crate::chunks::thm_thumbnail_chunk::*;
pub use crate::thm_bump_mode::*;
pub use crate::thm_bump_patch_report::*;
pub use crate::thm_bump_processor::*;
pub use crate::thm_detail_usage::*;
pub use crate::thm_file::*;
pub use crate::thm_format::*;
pub use crate::thm_material::*;
pub use crate::thm_mip_filter::*;
pub use crate::thm_texture_flag::*;
pub use crate::thm_texture_flags::*;
pub use crate::thm_texture_type::*;
