use xrf_level::BuildAimapFile;

use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_covers::AiMapCovers;
use crate::ray::ray_scene::RayScene;
use crate::ray::ray_tracing::RayTracing;

/// What [`crate::ai::ai_map_compiler::AiMapCompiler::compile`] compiles a level's AI map from, each already read.
#[derive(Clone, Copy)]
pub(crate) struct AiMapRequest<'a> {
  /// The nodes as the Build exported them, `build.aimap`.
  pub aimap: &'a BuildAimapFile,
  pub covers: AiMapCovers,
  /// What the covers are traced against: xrLC's ray model, `build.cform`; needed for [`AiMapCovers::Full`] alone.
  pub scene: Option<RayScene<'a>>,
  /// What traces the corner nodes' rays.
  pub tracing: RayTracing<'a>,
  /// The `level.ai` the compiled one replaces, whose identity it keeps while the graph is unchanged.
  pub replaced: Option<&'a AiMap>,
}
