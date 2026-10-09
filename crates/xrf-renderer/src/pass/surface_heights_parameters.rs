use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::surface_mask_uniform::SurfaceMaskUniform;

/// What taking the level surface's lowest heights reads and writes: its overhead depth, the heights, and its shape.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct SurfaceHeightsParameters {
  #[texture(d2, depth)]
  pub surface: GraphTexture,
  #[storage_texture(d2, r32float, write)]
  pub lowest: GraphTexture,
  #[uniform]
  pub shape: UniformBinding<SurfaceMaskUniform>,
}
