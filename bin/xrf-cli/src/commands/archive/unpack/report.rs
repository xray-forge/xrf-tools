//! What an `archive unpack --dry` run reports to a machine.
//!
//! A completed run reports `xrf_pack::ArchiveUnpackResult` directly: it is already this workspace's
//! shape for what unpacking produced, shared with the desktop app over IPC, and a second CLI-only
//! shape for the same operation is the drift the reporting contract exists to remove. Both branches
//! render their paths through `xrf_utils::format_path`, so `destination` means one thing either way.
//!
//! A dry run produced no such result, so what it reports is what it would have written: the same
//! summary it printed, saying plainly that nothing was.

use serde::Serialize;
use xrf_archive::ArchiveProject;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveUnpackDryReport {
  archives: usize,
  destination: String,
  entries: usize,
  is_dry: bool,
  size_compressed: u64,
  size_real: u64,
  source: String,
}

impl ArchiveUnpackDryReport {
  pub fn new(project: &ArchiveProject, source: &str, destination: &str) -> Self {
    Self {
      archives: project.archives.len(),
      destination: String::from(destination),
      entries: project.files.len(),
      is_dry: true,
      size_compressed: project.get_compressed_size(),
      size_real: project.get_real_size(),
      source: String::from(source),
    }
  }
}
