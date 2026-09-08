//! What a run packs: the walk or comparison that finds it, the engine-name table that decides what may be registered,
//! and the one place its payloads are read from.

mod archive_pack_entry;
mod archive_pack_name_collision;
mod archive_pack_name_table;
mod archive_pack_omissions;
mod archive_pack_payloads;
mod archive_pack_source;
mod archive_pack_source_collector;

pub(crate) use archive_pack_entry::{ArchivePackEntry, ArchivePackOrigin};
pub(crate) use archive_pack_name_collision::ArchivePackNameCollision;
pub(crate) use archive_pack_name_table::ArchivePackNameTable;
pub(crate) use archive_pack_omissions::ArchivePackOmissions;
pub(crate) use archive_pack_payloads::ArchivePackPayloads;
pub(crate) use archive_pack_source::ArchivePackSource;
pub(crate) use archive_pack_source_collector::ArchivePackSourceCollector;
