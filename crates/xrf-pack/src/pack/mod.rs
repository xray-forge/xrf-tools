//! Writing a source tree into the `.db` volume set the engine mounts.
//!
//! The run is grouped by the question each part answers:
//!
//! - [`config`] — what a run was asked to pack, and the two files that configuration is authored in.
//! - [`source`] — what the source tree holds, folded to the engine names one archive may register.
//! - [`volume`] — how selected entries become volumes, and what the destination already publishes.
//!
//! [`ArchivePacker`] is the operation itself, ordering those three; [`ArchivePackNarrator`] is the only place that
//! says what it decided, so a transcript of a run reads the same whichever phase produced the line.

pub(crate) mod config;
pub(crate) mod source;
pub(crate) mod volume;

mod archive_pack_decision;
mod archive_pack_narrator;
mod archive_pack_options;
mod archive_pack_result;
mod archive_packer;

#[cfg(test)]
mod tests;

pub(crate) use archive_pack_decision::{ArchivePackEntryOutcome, ArchivePackSkipReason};
pub(crate) use archive_pack_narrator::ArchivePackNarrator;
pub use archive_pack_options::{ArchivePackOptions, PACK_PHASE_COLLECT, PACK_PHASE_FINALIZE, PACK_PHASE_WRITE};
pub use archive_pack_result::ArchivePackResult;
pub use archive_packer::ArchivePacker;
