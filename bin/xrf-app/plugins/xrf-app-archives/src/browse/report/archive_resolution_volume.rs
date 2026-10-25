use std::path::PathBuf;

use serde::Serialize;
use xrf_archive::ArchiveDescriptor;

/// One volume of a set, in the order a lookup reaches it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResolutionVolume {
  /// The volume file itself.
  pub path: PathBuf,
  /// Entries its name table holds, before the merge shadows one of them.
  pub entries: usize,
  /// Bytes its entries occupy as stored.
  pub size_compressed: u64,
  /// Bytes its entries occupy once unpacked.
  pub size_real: u64,
  /// Root the volume unpacks under, from `[header] entry_point` with its alias stripped.
  pub output_root_path: PathBuf,
}

impl From<&ArchiveDescriptor> for ArchiveResolutionVolume {
  fn from(volume: &ArchiveDescriptor) -> Self {
    Self {
      entries: volume.entries,
      output_root_path: volume.output_root_path.clone(),
      path: volume.path.clone(),
      size_compressed: volume.size_compressed,
      size_real: volume.size_real,
    }
  }
}
