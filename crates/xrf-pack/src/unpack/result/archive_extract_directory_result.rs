use std::path::Path;

use serde::Serialize;
use xrf_job::JobOutcome;
use xrf_utils::format_path;

use crate::unpack::TreeWriteSummary;

/// What extracting one archived directory produced.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveExtractDirectoryResult {
  pub prefix: String,
  pub destination: String,
  /// Whether the run reached the end of what it selected or was stopped between entries.
  pub outcome: JobOutcome,
  pub extracted_count: usize,
  pub size: u64,
}

impl ArchiveExtractDirectoryResult {
  /// What a written selection under `prefix` came to below `destination`.
  pub(crate) fn of_summary(prefix: String, destination: &Path, summary: &TreeWriteSummary) -> Self {
    Self {
      prefix,
      destination: format_path(destination).to_string(),
      outcome: summary.outcome,
      extracted_count: summary.files,
      size: summary.bytes,
    }
  }

  /// A requested prefix in the form selection matches it in, without the trailing separator a person may type.
  pub(crate) fn normalize_prefix(prefix: &str) -> String {
    prefix.trim_end_matches(['\\', '/']).to_string()
  }
}
