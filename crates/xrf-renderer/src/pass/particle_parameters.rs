use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::particle_surface_record::ParticleSurfaceRecord;
use crate::pass::particle_vertex::ParticleVertex;

/// What the particle draws read: the quads' corners, the effects' surfaces, the frame's lighting and the scene's depth,
/// which they fade against.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct ParticleParameters<'a> {
  #[storage]
  pub vertices: StorageArray<ParticleVertex>,
  #[storage]
  pub surfaces: StorageArray<ParticleSurfaceRecord>,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[sampler(filtering)]
  pub clamped_sampler: &'a wgpu::Sampler,
}
