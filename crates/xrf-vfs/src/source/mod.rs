//! Mountable asset sources, and the indexing a directory source needs.

mod directory_asset;
mod directory_asset_index;
mod indexed_asset;
mod xray_archive_source;
mod xray_asset_index;
mod xray_asset_source;
mod xray_declared_root;
mod xray_directory_source;
mod xray_file_stamp;
mod xray_source_kind;

pub(crate) use directory_asset::DirectoryAsset;
pub(crate) use directory_asset_index::DirectoryAssetIndex;
pub(crate) use indexed_asset::IndexedAsset;
pub use xray_archive_source::XrayArchiveSource;
pub(crate) use xray_asset_index::XrayAssetIndex;
pub use xray_asset_source::{XrayAssetSource, label_from_path};
pub use xray_declared_root::XrayDeclaredRoot;
pub(crate) use xray_directory_source::XrayDirectorySource;
pub use xray_file_stamp::XrayFileStamp;
pub use xray_source_kind::XraySourceKind;
