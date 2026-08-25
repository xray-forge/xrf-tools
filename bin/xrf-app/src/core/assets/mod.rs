//! The mounted asset roots every domain resolves and reads through.

mod asset_mount_state;
mod asset_read;
mod asset_texture;

pub use asset_mount_state::AssetMountState;
pub use asset_read::read_located_asset;
pub use asset_texture::{AssetTextureDescriptor, read_texture_png};
