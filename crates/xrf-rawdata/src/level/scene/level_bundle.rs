use xrf_level::{LevelFile, LevelVisualsChunk};

/// The compiled bundle (`level`), read once for every step that needs it: the file, and its visuals where it holds any.
pub(crate) struct LevelBundle {
  pub(crate) file: LevelFile,
  pub(crate) visuals: Option<LevelVisualsChunk>,
}
