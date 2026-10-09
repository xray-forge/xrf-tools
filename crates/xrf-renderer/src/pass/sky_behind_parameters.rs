use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;

/// What the sky behind the fog is drawn from: the frame's lighting and exposure, the haze map, and the frame's depth,
/// which says where the fog shows it.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct SkyBehindParameters {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[texture(d2, float)]
  pub haze_map: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
}
