use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::feature::tracing::scene_trace::requests::water_rays_uniform::WaterRaysUniform;

/// The scene pass's way in from the water's every-pixel rays (`cs_water_scene`): their records (`WaterRays`), what they
/// are read by, and the nearest water's depth their starts are read back from. Group 6, past the list's, which these
/// rays leave empty.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 6)]
pub(crate) struct WaterRayParameters {
  #[storage]
  pub water_rays: StorageArray<u32>,
  #[uniform]
  pub water_rays_frame: UniformBinding<WaterRaysUniform>,
  #[texture(d2, depth)]
  pub water_depth: GraphTexture,
}
