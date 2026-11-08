use xrf_renderer_core::{GraphTexture, PassParameters, StorageArrayMut, UniformBinding};

use crate::feature::weather::water::enhanced::enhanced_water_uniform::EnhancedWaterUniform;
use crate::feature::weather::water::water_uniform::WaterUniform;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// What the enhanced water's reflection reads beside the scene: what every water program reads, the scene before the
/// water, the history the frames before accumulated, and the pyramid of the frame's nearest depth its rays walk over;
/// this frame's reflection, its traced hits written in, which the resolve keeps over the history; and the records it
/// leaves the scene pass while every pixel traces. Group 4, past the request list's.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 4)]
pub(crate) struct WaterReflectionParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub water: UniformBinding<WaterUniform>,
  #[uniform]
  pub enhanced: UniformBinding<EnhancedWaterUniform>,
  /// The G-buffer's depth, which the reflection is marched over.
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(cube, float)]
  pub sky_cube_0: GraphTexture,
  #[texture(cube, float)]
  pub sky_cube_1: GraphTexture,
  #[sampler(filtering)]
  pub sky_clamp: &'a wgpu::Sampler,
  /// The nearest water along each pixel, which the depth pass wrote: only that surface draws.
  #[texture(d2, depth)]
  pub nearest_water: GraphTexture,
  /// The scene as it stood before the water, which the reflection reads where its ray hit.
  #[texture(d2, float)]
  pub water_scene: GraphTexture,
  #[texture(d2, float)]
  pub reflection_history: GraphTexture,
  /// The frame's nearest depth reduced level by level, the grass in it (`WaterReflection::pyramid`).
  #[texture(d2, unfilterable)]
  pub nearest_depth: GraphTexture,
  /// The G-buffer's material, whose marks tell a plant: the walk meets a plant as the volume it stands for, and while
  /// fully traced the screen answers plants alone.
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub water_current: GraphTexture,
  /// The records every pixel leaves the scene pass while every pixel traces every frame (`WaterRays`), written by the
  /// reflection alone: the resolve binds a record of nothing.
  #[storage]
  pub water_rays: StorageArrayMut<u32>,
}
