use serde::{Deserialize, Serialize};

use crate::level::surfaces::level_compiler_shader_reason::LevelCompilerShaderReason;

/// The compiler shader a decompile chose for the surfaces of one engine shader showing one kind of evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCompilerShaderChoice {
  pub engine_shader: String,
  /// Whether its faces lie on collision faces.
  pub collision: bool,
  /// Whether its vertices store a baked colour in place of lightmap coordinates.
  pub vertex_lighting: bool,
  pub compiler_shader: String,
  pub reason: LevelCompilerShaderReason,
  /// How many surfaces the library pairs the engine shader with it, where it does.
  pub pairings: usize,
}
