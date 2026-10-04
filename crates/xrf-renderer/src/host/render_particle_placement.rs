use crate::host::render_particle_source::RenderParticleSource;

/// A particle system placed on a level: what it plays, and where.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderParticlePlacement {
  pub source: RenderParticleSource,
  /// Where it is placed in engine space, the space the simulation runs in, sixteen floats column by column.
  pub transform: [f32; 16],
}
