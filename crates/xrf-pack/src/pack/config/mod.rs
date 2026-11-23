//! What a packing run was asked to pack: the configuration, its selection rules, and the files it is authored in.
//!
//! The configuration is a document a surface edits, imports and exports; how a run behaves while packing it is
//! [`crate::pack::ArchivePackOptions`] instead.
//!
//! - [`codec`] — the LTX and JSON files a configuration is read from and written back to.
//! - [`header`] — the `[header]` every volume carries, and what makes one loadable.
//! - [`volume`] — how the volumes are written, shared with a patch's publication.

mod archive_pack_config;
mod archive_pack_config_rules;
mod archive_pack_directory;
mod codec;
mod header;
mod volume;

#[cfg(test)]
mod tests;

pub use archive_pack_config::ArchivePackConfig;
pub use archive_pack_directory::ArchivePackDirectory;
pub use codec::ArchivePackConfigJson;
pub use header::{ArchivePackHeader, ArchivePackHeaderEntry};
pub use volume::{ArchivePackMode, ArchiveVolumeExtension, ArchiveVolumeSettings};
