use xrf_level::BuildAimapFile;

use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_covers::AiMapCovers;
use crate::ai::cover::ai_ray_model::AiRayModel;

/// What [`crate::AiMapCompiler::compile`] compiles a level's AI map from, each already read.
#[derive(Clone, Copy)]
pub struct AiMapRequest<'a> {
  /// The nodes as the Build exported them, `build.aimap`.
  pub aimap: &'a BuildAimapFile,
  pub covers: AiMapCovers,
  /// What the covers are traced against; needed for [`AiMapCovers::Full`] alone.
  pub rays: Option<AiRayModel<'a>>,
  /// The `level.ai` the compiled one replaces, whose identity it keeps while the graph is unchanged.
  pub replaced: Option<&'a AiMap>,
}
