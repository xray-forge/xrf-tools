use serde::Serialize;
use xrf_archive::ArchiveProject;

use crate::commands::archive::info::archive_volume_report::ArchiveVolumeReport;

/// What `archive info` found.
///
/// Every volume is reported whatever the verbosity: a machine consumer has no `--verbose` to raise,
/// so the per-volume detail a human would have asked for belongs here unconditionally. Paths are
/// portable display strings, since serializing a `PathBuf` fails outright on a host name that is not
/// valid Unicode and a report describes a run rather than addressing one.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveInfoReport {
  directories: usize,
  files: usize,
  root: String,
  size_compressed: u64,
  size_real: u64,
  volumes: Vec<ArchiveVolumeReport>,
}

impl ArchiveInfoReport {
  pub fn new(project: &ArchiveProject) -> Self {
    Self {
      directories: project.count_directories(),
      files: project.count_files(),
      root: xrf_utils::to_portable_path_string(&project.root),
      size_compressed: project.get_compressed_size(),
      size_real: project.get_real_size(),
      volumes: project.archives.iter().map(ArchiveVolumeReport::new).collect(),
    }
  }
}
