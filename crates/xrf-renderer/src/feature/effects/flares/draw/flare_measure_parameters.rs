use xrf_renderer_core::{GraphTexture, PassParameters, StorageValueMut, UniformBinding};

use crate::feature::effects::flares::flare_uniform::FlareUniform;
use crate::feature::lighting::shadows::shadow_uniform::ShadowUniform;

/// What measuring how much of the sun shows reads and writes: the lens flare, its eased visibility, the frame's depth
/// for the rays on screen and the sun's shadow for those off it.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub(crate) struct FlareMeasureParameters<'a> {
  #[uniform]
  pub flares: UniformBinding<FlareUniform>,
  #[storage]
  pub state: StorageValueMut<[f32; 4]>,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2_array, depth)]
  pub shadow_maps: GraphTexture,
  #[uniform]
  pub shadows: UniformBinding<ShadowUniform>,
  /// The comparison the cascades' maps are filtered through (`SunPass::get_compare`).
  #[sampler(comparison)]
  pub shadow_compare: &'a wgpu::Sampler,
}
