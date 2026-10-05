use xrf_visual::ZoneSphere;

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
  /// Its zone's sphere in engine space, offset from where it is placed, by which Monolith stops the zone's idle effect
  /// while the camera stands far (`o_switch_2_slow`). None for a planted system and for a zone always fast.
  pub zone_sphere: Option<ZoneSphere>,
}
