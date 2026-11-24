use xrf_renderer_core::{GraphTexture, PassParameters, ShaderAtomicU32, StorageArray, StorageArrayMut, UniformBinding};

use crate::feature::tracing::scene_trace::requests::water_rays_uniform::WaterRaysUniform;

/// The scene pass's way in from the water's every-pixel rays (`cs_water_scene`): their records (`WaterRays`), how
/// they are read, and the nearest water's depth their starts are read back from. Group 6; the list's group 5 stays
/// empty.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 6)]
pub(crate) struct WaterRayParameters {
  #[storage]
  pub water_rays: StorageArray<u32>,
  #[uniform]
  pub water_rays_frame: UniformBinding<WaterRaysUniform>,
  #[texture(d2, depth)]
  pub water_depth: GraphTexture,
  /// How the trace's own light is brought to the screen's (`WaterRays::calibration`).
  #[storage]
  pub water_calibration: StorageArrayMut<ShaderAtomicU32>,
}
