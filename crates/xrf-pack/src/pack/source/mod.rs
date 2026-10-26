//! What a run packs: the walk or comparison that finds it, the engine-name table that decides what may be registered,
//! and the one place its payloads are read from.
//!
//! - [`names`] — the engine-name table a run registers, and the collisions it refuses.
//! - [`omissions`] — what the selection rules left out, and why.

mod archive_pack_entry;
mod archive_pack_origin;
mod archive_pack_payloads;
mod archive_pack_source;
mod archive_pack_source_collector;
mod names;
mod omissions;

pub(crate) use archive_pack_entry::ArchivePackEntry;
pub(crate) use archive_pack_origin::ArchivePackOrigin;
pub(crate) use archive_pack_payloads::ArchivePackPayloads;
pub(crate) use archive_pack_source::ArchivePackSource;
pub(crate) use archive_pack_source_collector::ArchivePackSourceCollector;
pub(crate) use names::{ArchivePackNameCollision, ArchivePackNameTable};
pub(crate) use omissions::{
  ArchivePackExcludedDirectory, ArchivePackOmissions, ArchivePackSkipReason, ArchivePackSkippedFile,
};
