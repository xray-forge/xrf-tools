#![doc = include_str!("../README.md")]

mod archive_config_format;
mod archive_config_ltx;
mod archive_extension_pattern;
#[cfg(test)]
mod asset_source_tests;
pub(crate) mod pack;
pub(crate) mod patch;
pub(crate) mod path;
pub(crate) mod unpack;

pub use crate::archive_config_format::ArchiveConfigFormat;
pub use crate::pack::config::{
  ArchivePackConfig, ArchivePackConfigJson, ArchivePackDirectory, ArchivePackHeaderEntry, ArchivePackMode,
  ArchiveVolumeExtension,
};
pub use crate::pack::{ArchivePackOptions, ArchivePackResult, ArchivePacker};
pub use crate::patch::compare::{ArchivePatchChange, ArchivePatchClass, ArchivePatchOrigin, ArchivePatchSide};
pub use crate::patch::config::{ArchivePatchConfig, ArchivePatchConfigJson};
pub use crate::patch::{ArchivePatchOptions, ArchivePatchPublication, ArchivePatchResult, ArchivePatcher};
pub use crate::unpack::{
  ArchiveExtractDirectoryResult, ArchiveExtractOptions, ArchiveExtractResult, ArchiveUnpackResult, ArchiveUnpacker,
  XrayWorldExtractor,
};
