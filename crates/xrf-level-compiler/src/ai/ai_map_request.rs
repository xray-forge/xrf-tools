use xrf_level::BuildAimapFile;

use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_covers::AiMapCovers;

/// What [`crate::AiMapCompiler::compile`] compiles a level's AI map from, each already read.
#[derive(Clone, Copy)]
pub struct AiMapRequest<'a> {
  /// The nodes as the Build exported them, `build.aimap`.
  pub aimap: &'a BuildAimapFile,
  pub covers: AiMapCovers,
  /// The `level.ai` the compiled one replaces, whose identity it keeps while the graph is unchanged.
  pub replaced: Option<&'a AiMap>,
}
