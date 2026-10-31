use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::pass::froxel_uniform::FroxelUniform;

/// The froxels as the frame's graph holds them once integrated: the grid's uniform as pushed, and what was integrated
/// along the view.
#[derive(Clone, Copy)]
pub struct Froxels {
  pub uniform: UniformBinding<FroxelUniform>,
  pub integrated: GraphTexture,
}
