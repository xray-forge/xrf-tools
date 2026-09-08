//! Comparing two mounted worlds and writing the difference as a volume set that overrides the first.

pub(crate) mod compare;
pub(crate) mod config;
pub(crate) mod world;

mod archive_patch_directory_rows;
mod archive_patch_narrator;
mod archive_patch_options;
mod archive_patch_publication;
mod archive_patch_result;
mod archive_patcher;

#[cfg(test)]
mod tests;

pub(crate) use archive_patch_directory_rows::to_patch_directory_rows;
pub(crate) use archive_patch_narrator::ArchivePatchNarrator;
pub use archive_patch_options::{ArchivePatchOptions, PATCH_PHASE_COMPARE, PATCH_PHASE_PACK};
pub use archive_patch_publication::ArchivePatchPublication;
pub use archive_patch_result::ArchivePatchResult;
pub use archive_patcher::ArchivePatcher;
