//! The mounted asset roots every domain resolves and reads through.

mod asset_change_event;
mod asset_change_order;
mod asset_change_order_state;
mod asset_change_subscriber;
mod asset_change_ticket;
mod asset_engine;
mod asset_fan_out;
mod asset_mount_state;
mod asset_read;
mod asset_texture_descriptor;
mod asset_texture_png;
mod asset_texture_shape;

pub use asset_change_event::AssetChangeEvent;
pub use asset_change_subscriber::AssetChangeSubscriber;
pub use asset_engine::{detect_roots_engine, detect_search_engine};
pub use asset_fan_out::AssetFanOut;
pub use asset_mount_state::AssetMountState;
pub use asset_read::{locate_referenced_asset, read_located_asset, read_referenced_asset};
pub use asset_texture_descriptor::AssetTextureDescriptor;
pub use asset_texture_png::read_texture_png;
pub use asset_texture_shape::AssetTextureShape;
