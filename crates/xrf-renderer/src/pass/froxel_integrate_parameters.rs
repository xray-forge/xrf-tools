use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::froxel_uniform::FroxelUniform;

/// What integrating the froxels reads and writes: the grid, what each froxel scatters and how thick it is, and what is
/// scattered towards the camera up to each.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct FroxelIntegrateParameters {
  #[uniform]
  pub froxels: UniformBinding<FroxelUniform>,
  #[texture(d3, float)]
  pub froxel_scattered: GraphTexture,
  #[storage_texture(d3, rgba16float, write)]
  pub froxel_integrated: GraphTexture,
}
