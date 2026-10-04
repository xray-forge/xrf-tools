use crate::pass::particle_blend::ParticleBlend;

/// A run of particle quads drawn with one equation, in the order the frame draws them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticleBatch {
  pub blend: ParticleBlend,
  /// The first quad, and how many.
  pub first: u32,
  pub count: u32,
}
