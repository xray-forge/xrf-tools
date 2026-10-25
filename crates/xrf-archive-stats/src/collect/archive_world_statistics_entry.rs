use xrf_vfs::XrayAssetContainer;

use crate::collect::archive_statistics_entry::ArchiveStatisticsEntry;

/// What a mounted world adds: where the winning copy sits, and the sized copies it hides.
pub trait ArchiveWorldStatisticsEntry: ArchiveStatisticsEntry {
  /// Where the winning copy physically sits.
  fn get_container(&self) -> &XrayAssetContainer;

  /// Copies of this engine path no lookup reaches, each with its own unpacked size.
  fn list_shadowed(&self) -> impl Iterator<Item = (&XrayAssetContainer, u64)>;
}
