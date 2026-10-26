//! The `[header]` chunk 666 carries into every volume: its entries, its default, and the rules the engine loads it by.

mod archive_pack_header;
mod archive_pack_header_entry;

pub(crate) use archive_pack_header::ArchivePackHeader;
pub use archive_pack_header_entry::ArchivePackHeaderEntry;
