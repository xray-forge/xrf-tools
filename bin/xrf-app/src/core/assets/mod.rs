//! The mounted asset world every domain resolves and reads through.

mod asset_texture;
mod asset_world;

pub use asset_texture::AssetTextureDescriptor;
pub use asset_world::{AssetWorldSpec, AssetWorldState};
