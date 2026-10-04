use crate::host::render_particle_source::RenderParticleSource;

/// A particle system placed on a level: what it plays, and where.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderParticlePlacement {
  pub source: RenderParticleSource,
  /// Where it is placed in engine space, the space the simulation runs in, sixteen floats column by column.
  pub transform: [f32; 16],
  /// The object motion carrying its zone, by name, which moves it each frame from the transform: a torrid zone's idle
  /// effect (`CTorridZone`). None for one that stands.
  pub motion: Option<String>,
}
