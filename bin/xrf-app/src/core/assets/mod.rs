//! The mounted asset roots every domain resolves and reads through.

mod asset_mount_state;
mod asset_read;
mod asset_texture_descriptor;
mod asset_texture_png;
mod asset_texture_shape;

pub use asset_mount_state::AssetMountState;
pub use asset_read::{read_located_asset, read_referenced_asset};
pub use asset_texture_descriptor::AssetTextureDescriptor;
pub use asset_texture_png::read_texture_png;
pub use asset_texture_shape::AssetTextureShape;
