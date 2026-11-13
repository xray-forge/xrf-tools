use std::path::Path;

use xrf_geometry::BvhTracer;
use xrf_level::{LevelDetailsFile, LevelLightsFile};

use crate::detail::detail_gpu::DetailGpu;
use crate::detail::detail_light_backend::DetailLightBackend;
use crate::shadow::shadow_materials::ShadowMaterials;

/// What [`crate::DetailLighter::light`] lights a level's details from, each already read.
#[derive(Clone, Copy)]
pub struct DetailLightRequest<'a> {
  /// The details as the Build exported them, `build.details`: the slots to light, their light zeroed.
  pub details: &'a LevelDetailsFile,
  /// xrLC's ray model, `build.cform`'s faces (`BuildCformFile::build_tracer`).
  pub tracer: &'a BvhTracer,
  /// How each of the ray model's faces meets a light ray.
  pub shadows: &'a ShadowMaterials,
  /// xrLC's light lists, `build.lights`.
  pub lights: &'a LevelLightsFile,
  /// What traces the shadow rays.
  pub backend: DetailLightBackend,
  /// DXC's `dxcompiler.dll`, with which the GPU backend traces on D3D12; Vulkan alone without it.
  pub dxc: Option<&'a Path>,
  /// The GPU the GPU backend traces on; the high-performance one without it.
  pub gpu: Option<DetailGpu>,
}
