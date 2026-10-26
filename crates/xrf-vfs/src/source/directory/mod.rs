//! A loose directory as a source: the walk over its files and the index keying them by engine identity.

mod directory_asset_index;
mod indexed_asset;
mod xray_asset_index;
mod xray_directory_source;

pub(crate) use directory_asset_index::DirectoryAssetIndex;
pub(crate) use indexed_asset::IndexedAsset;
pub(crate) use xray_asset_index::XrayAssetIndex;
pub(crate) use xray_directory_source::XrayDirectorySource;
