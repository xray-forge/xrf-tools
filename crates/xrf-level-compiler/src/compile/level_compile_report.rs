use serde::Serialize;

use crate::compile::level_compile_step_report::LevelCompileStepReport;

/// What a level's compile ran (`LevelCompiler::compile`), as the front-ends report it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompileReport {
  /// The level's directory.
  pub level: String,
  /// Each step that ran, in the order it ran.
  pub steps: Vec<LevelCompileStepReport>,
}
