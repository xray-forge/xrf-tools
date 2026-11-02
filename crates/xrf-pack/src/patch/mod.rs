//! Comparing two mounted worlds and writing the difference as a volume set that overrides the first.
//!
//! - [`config`] — what a run was asked to compare, and what it publishes the difference as.
//! - [`world`] — one side of the comparison, mounted the way the engine would see it.
//! - [`compare`] — what the two sides differ by, and the vocabulary that difference is reported in.
//!
//! [`ArchivePatcher`] is the run itself, publishing through the packer's volume writer; [`ArchivePatchNarrator`] says
//! what it decided.

pub(crate) mod compare;
pub(crate) mod config;
pub(crate) mod world;

mod archive_patch_narrator;
mod archive_patch_options;
mod archive_patch_publication;
mod archive_patch_result;
mod archive_patcher;

#[cfg(test)]
mod tests;

pub(crate) use archive_patch_narrator::ArchivePatchNarrator;
pub use archive_patch_options::ArchivePatchOptions;
pub use archive_patch_publication::ArchivePatchPublication;
pub use archive_patch_result::ArchivePatchResult;
pub use archive_patcher::ArchivePatcher;
