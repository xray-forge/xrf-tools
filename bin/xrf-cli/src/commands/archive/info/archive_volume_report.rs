use serde::Serialize;
use xrf_archive::ArchiveDescriptor;

/// One volume of a set.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVolumeReport {
  entries: usize,
  path: String,
  root: String,
  size_compressed: u64,
  size_real: u64,
}

impl ArchiveVolumeReport {
  pub fn new(archive: &ArchiveDescriptor) -> Self {
    Self {
      entries: archive.entries,
      path: xrf_utils::to_portable_path_string(&archive.path),
      root: xrf_utils::to_portable_path_string(&archive.output_root_path),
      size_compressed: archive.get_compressed_size(),
      size_real: archive.get_real_size(),
    }
  }
}
