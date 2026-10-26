//! Mountable asset sources: the surface a mount reads, and the two sources the engine itself has.

mod directory;
mod xray_archive_source;
mod xray_asset_source;
mod xray_declared_root;
mod xray_file_stamp;
mod xray_source_kind;

pub(crate) use directory::XrayDirectorySource;
pub use xray_archive_source::XrayArchiveSource;
pub use xray_asset_source::{XrayAssetSource, label_from_path};
pub use xray_declared_root::XrayDeclaredRoot;
pub use xray_file_stamp::XrayFileStamp;
pub use xray_source_kind::XraySourceKind;
