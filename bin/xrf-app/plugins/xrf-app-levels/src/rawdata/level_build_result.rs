use serde::Serialize;
use xrf_job::JobOutcome;
use xrf_level_compiler::LevelCompileReport;
use xrf_rawdata::LevelBuildReport;

/// How a scene's build, and the compile after it, ended: the build's report once it finished, and the compile's once
/// one was asked for and finished.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelBuildResult {
  pub(crate) outcome: JobOutcome,
  pub(crate) report: Option<LevelBuildReport>,
  pub(crate) compile: Option<LevelCompileReport>,
}
