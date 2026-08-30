use std::path::PathBuf;
use std::time::Duration;

use serde::Serialize;
use xrf_job::JobOutcome;
use xrf_utils::format_path;

use crate::LtxFormatOptions;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxProjectFormatResult {
  /// Whether the run reached the end of the set or was stopped between files.
  ///
  /// A stopped rewrite leaves the files it had already formatted formatted and the rest untouched, which is a state
  /// running it again resolves. Nothing is removed and nothing is half-written.
  pub outcome: JobOutcome,

  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  pub invalid_files: usize,
  pub to_format: Vec<PathBuf>,
  pub total_files: usize,
  pub valid_files: usize,
}

impl LtxProjectFormatResult {
  pub fn new() -> Self {
    Self {
      outcome: JobOutcome::Completed,
      duration: Duration::ZERO,
      invalid_files: 0,
      to_format: Vec::new(),
      total_files: 0,
      valid_files: 0,
    }
  }

  /// Records the verdict for one checked config and reports it.
  ///
  /// Shared by the file-based formatter and the project check, which differ only in how they obtain the contents.
  pub(crate) fn record_checked(&mut self, path: PathBuf, is_formatted: bool, options: &LtxFormatOptions) {
    if is_formatted {
      self.valid_files += 1;
    } else {
      xrf_output::info!(options.output, "Not formatted: {}", format_path(&path));

      self.invalid_files += 1;
      self.to_format.push(path);
    }

    self.total_files += 1;
  }
}
