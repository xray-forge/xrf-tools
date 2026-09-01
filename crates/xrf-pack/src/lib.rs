#![doc = include_str!("../README.md")]
//!
//! Both operations own no state and borrow the project, so neither is a method on it.

#[cfg(test)]
mod asset_source_tests;
pub(crate) mod pack;
pub(crate) mod path;
pub(crate) mod unpack;

pub use crate::pack::archive_pack_config::{
  ArchivePackConfig, ArchivePackDirectory, ArchivePackMode, ArchiveVolumeExtension, VOLUME_SIZE_MAX, VOLUME_SIZE_MIN,
};
pub use crate::pack::archive_pack_options::{
  ArchivePackOptions, PACK_PHASE_COLLECT, PACK_PHASE_FINALIZE, PACK_PHASE_WRITE,
};
pub use crate::pack::archive_pack_result::ArchivePackResult;
pub use crate::pack::archive_packer::ArchivePacker;
pub use crate::unpack::archive_extract_options::{ArchiveExtractOptions, EXTRACT_PHASE_WRITE};
pub use crate::unpack::archive_extract_result::{ArchiveExtractDirectoryResult, ArchiveExtractResult};
pub use crate::unpack::archive_unpack_options::{ArchiveUnpackOptions, UNPACK_PHASE_PREPARE, UNPACK_PHASE_WRITE};
pub use crate::unpack::archive_unpack_result::ArchiveUnpackResult;
pub use crate::unpack::archive_unpacker::ArchiveUnpacker;
