//! What a packing run was asked to pack: the configuration, its selection rules, and the files it is authored in.
//!
//! The configuration is a document a surface edits, imports and exports; how a run behaves while packing it is
//! [`crate::pack::ArchivePackOptions`] instead.
//!
//! - [`codec`] — the LTX and JSON files a configuration is read from and written back to.
//! - [`header`] — the `[header]` every volume carries, and what makes one loadable.

mod archive_pack_config;
mod archive_pack_config_rules;
mod archive_pack_directory;
mod archive_pack_mode;
mod archive_volume_extension;
mod codec;
mod header;

#[cfg(test)]
mod tests;

pub use archive_pack_config::ArchivePackConfig;
pub use archive_pack_directory::ArchivePackDirectory;
pub use archive_pack_mode::ArchivePackMode;
pub use archive_volume_extension::ArchiveVolumeExtension;
pub use codec::ArchivePackConfigJson;
pub(crate) use header::ArchivePackHeader;
pub use header::ArchivePackHeaderEntry;
