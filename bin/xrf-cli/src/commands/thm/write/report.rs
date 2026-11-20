use std::path::Path;

use serde::Serialize;
use xrf_thm::ThmFile;

/// What `thm write` described a texture as, and where it put the descriptor.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ThmWriteReport {
  descriptor: ThmFile,
  destination: String,
  is_dry_run: bool,
  source: String,
}

impl ThmWriteReport {
  pub(crate) fn new(source: &Path, destination: &Path, descriptor: ThmFile, is_dry_run: bool) -> Self {
    Self {
      descriptor,
      destination: xrf_utils::format_portable_path(destination),
      is_dry_run,
      source: xrf_utils::format_portable_path(source),
    }
  }
}
