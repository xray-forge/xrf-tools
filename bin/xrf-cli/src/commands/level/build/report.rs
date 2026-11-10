use serde::Serialize;
use xrf_rawdata::{LevelBuildCensus, LevelExportedFile};

/// What `level build` wrote, as its JSON report says it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelBuildReport {
  pub(crate) source: String,
  pub(crate) destination: String,
  /// Each file written, its size and what it replaced.
  pub(crate) files: Vec<LevelExportedFile>,
  /// What `build.prj` took from the scene, where the build wrote one.
  pub(crate) geometry: Option<LevelBuildCensus>,
}
