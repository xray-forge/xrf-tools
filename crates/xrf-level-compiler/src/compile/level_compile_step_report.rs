use serde::Serialize;

use crate::compile::level_compile_step::LevelCompileStep;
use crate::compile::level_compile_step_run::LevelCompileStepRun;

/// One step a compile ran.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompileStepReport {
  pub step: LevelCompileStep,
  /// How long it ran, in milliseconds.
  pub duration: u64,
  pub run: LevelCompileStepRun,
}
