//! The mounted asset world every domain resolves and reads through.

mod asset_read;
mod asset_texture;
mod asset_world_state;

pub use asset_read::read_located_asset;
pub use asset_texture::AssetTextureDescriptor;
pub use asset_world_state::AssetWorldState;
