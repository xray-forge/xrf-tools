//! Writing a source tree into the `.db` volume set the engine mounts.
//!
//! The run is grouped by the question each part answers:
//!
//! - [`config`] — what a run was asked to pack, and the two files that configuration is authored in.
//! - [`source`] — what the source tree holds, folded to the engine names one archive may register.
//! - [`volume`] — how selected entries become volumes, and what the destination already publishes.
//!
//! [`ArchivePacker`] is the operation itself, ordering those three; it prints nothing and hands its decisions out as
//! [`events`], which [`events::ArchivePackPrinter`] says, so a transcript reads the same whichever phase decided.

pub(crate) mod config;
pub(crate) mod events;
pub(crate) mod source;
pub(crate) mod volume;

mod archive_pack_entry_outcome;
mod archive_pack_options;
mod archive_pack_result;
mod archive_packer;

#[cfg(test)]
mod tests;

pub use archive_pack_entry_outcome::ArchivePackEntryOutcome;
pub use archive_pack_options::ArchivePackOptions;
pub use archive_pack_result::ArchivePackResult;
pub use archive_packer::ArchivePacker;
