use serde::Serialize;

use crate::level_compile_step::LevelCompileStep;
use crate::level_compile_step_run::LevelCompileStepRun;

/// What a level's compile ran ([`crate::LevelCompiler::compile`]), as the front-ends report it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompileReport {
  pub level: String,
  /// The directory the SDK's compilers ran in, with their `fsgame.ltx` and logs; none where no step ran one.
  pub work: Option<String>,
  pub steps: Vec<LevelCompileStepReport>,
}

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
