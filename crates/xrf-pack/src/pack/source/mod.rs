//! What the source tree holds: the walk that finds it, and the engine-name table that decides what may be registered.
//!
//! Discovery and registration are deliberately two seams. The walk answers what exists and what the rules want;
//! the table answers what the engine could address, and refuses a set it would fold into fewer files.

mod archive_pack_entry;
mod archive_pack_name_collision;
mod archive_pack_name_table;
mod archive_pack_omissions;
mod archive_pack_source;
mod archive_pack_source_collector;

pub(crate) use archive_pack_entry::ArchivePackEntry;
pub(crate) use archive_pack_name_collision::ArchivePackNameCollision;
pub(crate) use archive_pack_name_table::ArchivePackNameTable;
pub(crate) use archive_pack_omissions::ArchivePackOmissions;
pub(crate) use archive_pack_source::ArchivePackSource;
pub(crate) use archive_pack_source_collector::ArchivePackSourceCollector;
