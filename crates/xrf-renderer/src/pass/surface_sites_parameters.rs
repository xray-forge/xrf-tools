use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::surface_mask_uniform::SurfaceMaskUniform;

/// What placing puddle sites on the level surface reads and writes: its shape, its lowest heights, its water, and a
/// site a cell.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct SurfaceSitesParameters {
  #[binding(2)]
  #[uniform]
  pub shape: UniformBinding<SurfaceMaskUniform>,
  #[texture(d2, unfilterable)]
  pub heights: GraphTexture,
  #[texture(d2, depth)]
  pub water: GraphTexture,
  #[storage_texture(d2, rgba32float, write)]
  pub sites: GraphTexture,
}
