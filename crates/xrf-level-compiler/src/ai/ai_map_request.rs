use std::path::Path;

use xrf_level::BuildAimapFile;

use crate::ai::ai_map::AiMap;
use crate::ai::ai_map_covers::AiMapCovers;
use crate::ai::cover::ai_ray_model::AiRayModel;
use crate::ray::ray_backend::RayBackend;
use crate::ray::ray_gpu::RayGpu;

/// What [`crate::AiMapCompiler::compile`] compiles a level's AI map from, each already read.
#[derive(Clone, Copy)]
pub struct AiMapRequest<'a> {
  /// The nodes as the Build exported them, `build.aimap`.
  pub aimap: &'a BuildAimapFile,
  pub covers: AiMapCovers,
  /// What the covers are traced against; needed for [`AiMapCovers::Full`] alone.
  pub rays: Option<AiRayModel<'a>>,
  /// What traces the corner nodes' rays, as it traces the detail lighter's.
  pub backend: RayBackend,
  /// DXC's `dxcompiler.dll`, with which the GPU backend traces on D3D12; Vulkan alone without it.
  pub dxc: Option<&'a Path>,
  /// The GPU the GPU backend traces on; the high-performance one without it.
  pub gpu: Option<RayGpu>,
  /// The `level.ai` the compiled one replaces, whose identity it keeps while the graph is unchanged.
  pub replaced: Option<&'a AiMap>,
}
