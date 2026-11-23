//! How a set of volumes is written, apart from what goes in it: the payload mode, the extension, and the settings both
//! a pack and a patch publish volumes with.

mod archive_pack_mode;
mod archive_volume_extension;
mod archive_volume_settings;

pub use archive_pack_mode::ArchivePackMode;
pub use archive_volume_extension::ArchiveVolumeExtension;
pub use archive_volume_settings::ArchiveVolumeSettings;
