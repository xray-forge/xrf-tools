//! Where and how the renderer loads a texture, which both a descriptor and a surface are read within.

mod xray_reduced_textures;
mod xray_texture_scope;

pub use xray_reduced_textures::XrayReducedTextures;
pub use xray_texture_scope::XrayTextureScope;
