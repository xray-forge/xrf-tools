//! The mounted asset roots every domain resolves and reads through.

mod asset_mount_state;
mod asset_source;
mod change;
mod file_stamp;
#[cfg(test)]
mod tests;
mod texture;

pub use asset_mount_state::AssetMountState;
pub use asset_source::AssetSource;
pub use change::{AssetChangeEvent, AssetChangeSubscriber, AssetFanOut};
pub use file_stamp::FileStamp;
pub use texture::{AssetTextureDescriptor, AssetTextureImage, AssetTextureShape};
