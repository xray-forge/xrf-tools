//! What a run was asked to compare, and what it publishes the difference as.

mod archive_patch_config;
mod archive_patch_scope;
mod archive_patch_shape;

pub use archive_patch_config::ArchivePatchConfig;
pub(crate) use archive_patch_scope::ArchivePatchScope;
pub use archive_patch_shape::ArchivePatchShape;
