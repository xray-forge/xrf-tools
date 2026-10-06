use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::shadow_uniform::ShadowUniform;

/// What the sun's shafts read: the depth, the sun's shadow cascades, the frame's lighting and exposure, and the
/// half-size march the composite brings up, nothing while it is marched.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct SunShaftsParameters {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2_array, depth)]
  pub shadow_maps: GraphTexture,
  #[uniform]
  pub shadows: UniformBinding<ShadowUniform>,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[texture(d2, unfilterable)]
  pub shafts: GraphTexture,
}
