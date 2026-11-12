use serde::Serialize;

use crate::level_compile_step::LevelCompileStep;

/// What a level's compile ran ([`crate::LevelCompiler::compile`]), as the front-ends report it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompileReport {
  pub level: String,
  /// The directory the compilers ran in, with their `fsgame.ltx` and logs.
  pub work: String,
  pub steps: Vec<LevelCompileStepReport>,
}

/// One step a compile ran.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompileStepReport {
  pub step: LevelCompileStep,
  /// The program that ran it.
  pub program: String,
  /// How long it ran, in milliseconds.
  pub duration: u64,
  /// The compiler's log, where it wrote one.
  pub log: Option<String>,
}
