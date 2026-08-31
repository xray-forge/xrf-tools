use serde::Serialize;
use xrf_job::JobOutcome;

/// What extracting one archived file produced.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveExtractResult {
  pub name: String,
  pub destination: String,
  pub size: u64,
}

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
