use xrf_level::LevelShadersChunk;

use crate::Finding;

/// Findings from the binary level files, plus the shader table left for reference closure.
#[derive(Default)]
pub(crate) struct LevelBinariesOutcome {
  pub(crate) findings: Vec<Finding>,
  pub(crate) shaders: Option<LevelShadersChunk>,
}
