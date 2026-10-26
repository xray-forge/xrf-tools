#![doc = include_str!("../README.md")]

pub(crate) mod bump;
pub(crate) mod chunks;
pub(crate) mod object;
pub(crate) mod sound;
pub(crate) mod texture;
pub(crate) mod thm_file;
pub(crate) mod thm_kind;

#[cfg(test)]
mod tests;

pub use crate::bump::thm_bump_mode::ThmBumpMode;
pub use crate::bump::thm_bump_patch_report::ThmBumpPatchReport;
pub use crate::bump::thm_bump_processor::ThmBumpProcessor;
pub use crate::chunks::thm_bump_chunk::ThmBumpChunk;
pub use crate::chunks::thm_detail_chunk::ThmDetailChunk;
pub use crate::chunks::thm_extra_chunk::ThmExtraChunk;
pub use crate::chunks::thm_material_chunk::ThmMaterialChunk;
pub use crate::chunks::thm_texture_param_chunk::ThmTextureParamChunk;
pub use crate::chunks::thm_thumbnail_chunk::ThmThumbnailChunk;
pub use crate::chunks::thm_trailing_bytes::ThmTrailingBytes;
pub use crate::object::thm_group_file::ThmGroupFile;
pub use crate::object::thm_object_file::ThmObjectFile;
pub use crate::sound::thm_sound_file::ThmSoundFile;
pub use crate::texture::thm_detail_usage::ThmDetailUsage;
pub use crate::texture::thm_format::ThmFormat;
pub use crate::texture::thm_material::ThmMaterial;
pub use crate::texture::thm_mip_filter::ThmMipFilter;
pub use crate::texture::thm_texture_flag::ThmTextureFlag;
pub use crate::texture::thm_texture_flags::ThmTextureFlags;
pub use crate::texture::thm_texture_type::ThmTextureType;
pub use crate::thm_file::ThmFile;
pub use crate::thm_kind::ThmKind;
