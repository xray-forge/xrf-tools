//! What a resolved asset is, and the per-kind rules for turning an engine reference into one.

mod shader;
mod sound;
mod xray_asset;
mod xray_asset_container;
mod xray_asset_rules;
mod xray_asset_type;

pub use xray_asset::{XrayAsset, require_writable_path};
pub use xray_asset_container::XrayAssetContainer;
pub use xray_asset_rules::XrayAssetRules;
pub use xray_asset_type::XrayAssetType;
