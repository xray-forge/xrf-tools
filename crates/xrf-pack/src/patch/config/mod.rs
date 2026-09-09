//! What a run was asked to compare, and what it publishes the difference as.

mod archive_patch_config;
mod archive_patch_config_format;
mod archive_patch_config_json;
mod archive_patch_config_ltx;
mod archive_patch_scope;

pub use archive_patch_config::ArchivePatchConfig;
pub use archive_patch_config_format::ArchivePatchConfigFormat;
pub use archive_patch_config_json::ArchivePatchConfigJson;
pub(crate) use archive_patch_scope::ArchivePatchScope;
