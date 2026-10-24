use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_pack::{
  ArchivePackConfig, ArchivePackHeaderEntry, ArchivePackMode, ArchivePatchConfig, ArchiveVolumeExtension,
};

/// The volumes a pack or a patch was asked to write: where, under what name and header, how stored, how large, and
/// whether a set already there may be replaced.
pub struct ArchiveVolumeRequest {
  pub dest: PathBuf,
  pub name: String,
  /// The header entries named, merged over the default header, or none where none was named.
  pub header: Option<Vec<ArchivePackHeaderEntry>>,
  pub is_stored: bool,
  pub is_xdb: bool,
  /// The volume cap in bytes, where one was named.
  pub max_volume_size: Option<u64>,
  pub is_oversized_allowed: bool,
  pub is_forced: bool,
}

impl ArchiveVolumeRequest {
  /// `config` with these volumes' settings applied over it.
  ///
  /// # Errors
  ///
  /// Returns an error for a volume size the configuration refuses.
  pub fn apply_to_pack(&self, mut config: ArchivePackConfig) -> XrfResult<ArchivePackConfig> {
    if let Some(header) = &self.header {
      config = config.with_header_entries(header);
    }

    if self.is_stored {
      config.mode = ArchivePackMode::Store;
    }

    if self.is_xdb {
      config.volume_extension = ArchiveVolumeExtension::Xdb;
    }

    // Before the size it lifts the bound for, which is the order `with_max_volume_size` reads them in.
    config = config.with_oversized_volumes(self.is_oversized_allowed);

    match self.max_volume_size {
      Some(size) => config.with_max_volume_size(size),
      None => Ok(config),
    }
  }

  /// `config` with these volumes' settings applied over it.
  ///
  /// # Errors
  ///
  /// Returns an error for a volume size the configuration refuses.
  pub fn apply_to_patch(&self, mut config: ArchivePatchConfig) -> XrfResult<ArchivePatchConfig> {
    if let Some(header) = &self.header {
      config.header = Some(ArchivePackHeaderEntry::join(header));
    }

    if self.is_stored {
      config.mode = ArchivePackMode::Store;
    }

    if self.is_xdb {
      config.volume_extension = ArchiveVolumeExtension::Xdb;
    }

    config = config.with_oversized_volumes(self.is_oversized_allowed);

    match self.max_volume_size {
      Some(size) => config.with_max_volume_size(size),
      None => Ok(config),
    }
  }
}
