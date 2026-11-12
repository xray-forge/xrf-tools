use serde::Serialize;
use xrf_job::JobOutcome;
use xrf_rawdata::LevelDecompileReport;

/// How a level's decompile into a scene ended: what it wrote where it completed, none where it was cancelled first.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelDecompileResult {
  pub(crate) outcome: JobOutcome,
  pub(crate) report: Option<LevelDecompileReport>,
}
