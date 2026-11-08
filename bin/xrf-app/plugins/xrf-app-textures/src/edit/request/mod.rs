//! What the editing commands were asked to do.

mod texture_bump_gloss;
mod texture_descriptor_save;
mod texture_encoding_save;
mod textures_build_request;
mod textures_make_bump_request;
mod textures_save_request;

pub(crate) use texture_bump_gloss::TextureBumpGloss;
pub(crate) use texture_descriptor_save::TextureDescriptorSave;
pub(crate) use texture_encoding_save::TextureEncodingSave;
pub(crate) use textures_build_request::TexturesBuildRequest;
pub(crate) use textures_make_bump_request::TexturesMakeBumpRequest;
pub(crate) use textures_save_request::TexturesSaveRequest;
