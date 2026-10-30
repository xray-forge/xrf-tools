use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::camera_effects_uniform::CameraEffectsUniform;

/// What a stage of the camera effects reads, every stage the one layout: the frame it works on and a second image
/// beside it, the distance and motion under each shown pixel, the tiles, and the drawn depth and motion, which only
/// the inputs stage reads, over the drawn texels under each pixel.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct CameraEffectsParameters<'a> {
  #[uniform]
  pub camera_effects: UniformBinding<CameraEffectsUniform>,
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[texture(d2, float)]
  pub beside: GraphTexture,
  #[texture(d2, float)]
  pub inputs: GraphTexture,
  #[texture(d2, float)]
  pub tiles: GraphTexture,
  #[sampler(filtering)]
  pub source_sampler: &'a wgpu::Sampler,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
}
