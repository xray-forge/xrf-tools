//! What the editing commands were asked to do.

mod texture_bump_gloss;
mod texture_descriptor_save;
mod texture_encoding_save;
mod textures_build_request;
mod textures_make_bump_request;
mod textures_save_request;

pub use texture_bump_gloss::TextureBumpGloss;
pub use texture_descriptor_save::TextureDescriptorSave;
pub use texture_encoding_save::TextureEncodingSave;
pub use textures_build_request::TexturesBuildRequest;
pub use textures_make_bump_request::TexturesMakeBumpRequest;
pub use textures_save_request::TexturesSaveRequest;
