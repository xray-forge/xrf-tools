//! What a packing run was asked to pack: the configuration, its selection rules, and the files it is authored in.
//!
//! The configuration is a document a surface edits, imports and exports; how a run behaves while packing it is
//! [`crate::pack::ArchivePackOptions`] instead.

mod archive_pack_config;
mod archive_pack_config_codec;
mod archive_pack_config_format;
mod archive_pack_config_json;
mod archive_pack_config_ltx;
mod archive_pack_config_rules;
mod archive_pack_header_entry;

#[cfg(test)]
mod tests;

pub use archive_pack_config::{
  ArchivePackConfig, ArchivePackDirectory, ArchivePackMode, ArchiveVolumeExtension, VOLUME_SIZE_HARD_MAX,
  VOLUME_SIZE_MAX, VOLUME_SIZE_MIN,
};
pub use archive_pack_config_format::ArchivePackConfigFormat;
pub use archive_pack_config_json::ArchivePackConfigJson;
pub use archive_pack_header_entry::ArchivePackHeaderEntry;
