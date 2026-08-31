use std::time::Duration;

use serde::Serialize;
use xrf_error::XrfError;
use xrf_job::JobOutcome;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxProjectVerifyResult {
  /// Whether the run reached the end of the project or was stopped between files.
  ///
  /// A stopped run reports the findings it had reached, so this is what separates "these are the problems" from
  /// "these are the problems found so far" - the one way a partial check can mislead.
  pub outcome: JobOutcome,
  pub checked_fields: usize,
  pub checked_sections: usize,
  /// Everything the run took, measured from when its caller created the job handle.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  /// How much of `duration` had already passed when the per-file work began.
  ///
  /// Mounting the roots, indexing the virtual filesystem, assembling the project and resolving its includes all happen
  /// before a single file is read, and on a cold filesystem they dominate: a run reporting only its own loop told the
  /// user one second where they had waited fifteen. Named rather than folded away, so the split stays readable.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub startup_duration: Duration,
  pub errors: Vec<XrfError>,
  pub invalid_sections: usize,
  pub skipped_sections: usize,
  pub total_files: usize,
  pub total_sections: usize,
  pub valid_sections: usize,
}

impl LtxProjectVerifyResult {
  pub fn new() -> Self {
    Self {
      outcome: JobOutcome::Completed,
      checked_fields: 0,
      checked_sections: 0,
      duration: Duration::ZERO,
      startup_duration: Duration::ZERO,
      errors: Vec::new(),
      invalid_sections: 0,
      skipped_sections: 0,
      total_files: 0,
      total_sections: 0,
      valid_sections: 0,
    }
  }
}
