//! Writing what a volume set or a mounted world holds back out to a directory.
//!
//! [`ArchiveUnpacker`] reads an archive project and [`XrayWorldExtractor`] a mounted search; both select what they
//! write before writing any of it, and hand the selection to one tree writer.
//!
//! - [`tree`] — the selected entries and the writer that lays them out below a destination root.
//! - [`result`] — what an unpack or an extraction reports.

mod archive_extract_options;
mod archive_unpacker;
mod result;
mod tree;
mod xray_world_extractor;

#[cfg(test)]
mod tests;

pub use archive_extract_options::ArchiveExtractOptions;
pub use archive_unpacker::ArchiveUnpacker;
pub use result::{ArchiveExtractDirectoryResult, ArchiveExtractResult, ArchiveUnpackResult};
pub(crate) use tree::{TreeEntry, TreeWriteSummary, TreeWriter};
pub use xray_world_extractor::XrayWorldExtractor;
