use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::froxel_uniform::FroxelUniform;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::shadow_uniform::ShadowUniform;

/// What the sun's shafts read: the depth, the sun's shadow cascades, the frame's lighting and exposure, the half-size
/// march the composite brings up, nothing while it is marched or where the sun's shafts are not drawn, and the local
/// lights' shafts the composite adds with them, nothing where none draws any; or the enhanced volume's froxels in place
/// of both, integrated along the view, a volume of one where none is drawn; and the frame as it stood before them, which
/// the authored levels' composite writes over from (`SunShaftsDraw::Over`), nothing elsewhere.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct SunShaftsParameters<'a> {
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
  #[texture(d2, unfilterable)]
  pub light_volumes: GraphTexture,
  #[texture(d2, unfilterable)]
  pub scene_before: GraphTexture,
  #[uniform]
  pub froxels: UniformBinding<FroxelUniform>,
  #[texture(d3, float)]
  pub froxel_integrated: GraphTexture,
  #[sampler(filtering)]
  pub froxel_sampler: &'a wgpu::Sampler,
}
