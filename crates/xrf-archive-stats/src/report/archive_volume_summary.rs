use std::path::PathBuf;

use serde::Serialize;
use xrf_archive::ArchiveDescriptor;

/// One volume of a set, as its own name table recorded it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVolumeSummary {
  /// The volume file this summary describes.
  pub path: PathBuf,
  /// Entries this volume's name table holds, directories included.
  pub entries: u64,
  /// Bytes its entries occupy as stored.
  pub size_compressed: u64,
  /// Bytes they occupy once unpacked.
  pub size_real: u64,
  /// Volume file modification time in Unix milliseconds, when the filesystem reports one.
  pub modified_at: Option<u64>,
}

impl From<&ArchiveDescriptor> for ArchiveVolumeSummary {
  fn from(descriptor: &ArchiveDescriptor) -> Self {
    Self {
      path: descriptor.path.clone(),
      entries: descriptor.entries as u64,
      size_compressed: descriptor.size_compressed,
      size_real: descriptor.size_real,
      modified_at: descriptor.modified_at,
    }
  }
}
