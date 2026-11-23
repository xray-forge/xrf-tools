use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_pack::{ArchivePackHeader, ArchivePackMode, ArchiveVolumeExtension, ArchiveVolumeSettings};

/// The volumes a pack or a patch was asked to write: where, under what name and header, how stored, how large, and
/// whether a set already there may be replaced.
pub(crate) struct ArchiveVolumeRequest {
  pub dest: PathBuf,
  pub name: String,
  /// The header entries named, merged over the default header, or none where none was named.
  pub header: Option<ArchivePackHeader>,
  pub is_stored: bool,
  pub is_xdb: bool,
  /// The volume cap in bytes, where one was named.
  pub max_volume_size: Option<u64>,
  pub is_oversized_allowed: bool,
  pub is_forced: bool,
}

impl ArchiveVolumeRequest {
  /// `settings` with what this request names applied over them, the same for a pack and a patch.
  ///
  /// # Errors
  ///
  /// Returns an error for a volume size the settings refuse.
  pub(crate) fn apply_to(&self, mut settings: ArchiveVolumeSettings) -> XrfResult<ArchiveVolumeSettings> {
    if let Some(header) = &self.header {
      settings.header = Some(header.clone());
    }

    if self.is_stored {
      settings.mode = ArchivePackMode::Store;
    }

    if self.is_xdb {
      settings.volume_extension = ArchiveVolumeExtension::Xdb;
    }

    // Before the size it lifts the bound for, which is the order `with_max_volume_size` reads them in.
    settings = settings.with_oversized_volumes(self.is_oversized_allowed);

    match self.max_volume_size {
      Some(size) => settings.with_max_volume_size(size),
      None => Ok(settings),
    }
  }
}
