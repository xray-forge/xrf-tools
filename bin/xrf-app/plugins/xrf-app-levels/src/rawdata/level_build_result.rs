use serde::Serialize;
use xrf_job::JobOutcome;
use xrf_rawdata::LevelBuildReport;

/// How a scene's build into a level directory ended: what it wrote where it completed, none where it was cancelled
/// before writing.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelBuildResult {
  pub(crate) outcome: JobOutcome,
  pub(crate) report: Option<LevelBuildReport>,
}
