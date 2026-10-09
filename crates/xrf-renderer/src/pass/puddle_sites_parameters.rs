use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::puddle_sites_uniform::PuddleSitesUniform;

/// What placing puddle sites on the level surface reads and writes: its shape, its lowest heights, its water, and a
/// site a cell.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct PuddleSitesParameters {
  #[binding(2)]
  #[uniform]
  pub shape: UniformBinding<PuddleSitesUniform>,
  #[texture(d2, unfilterable)]
  pub heights: GraphTexture,
  #[texture(d2, depth)]
  pub water: GraphTexture,
  #[storage_texture(d2, rgba32float, write)]
  pub sites: GraphTexture,
}
