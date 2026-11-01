use crate::feature::effects::particles::particle_blend::ParticleBlend;
use crate::feature::effects::particles::particle_surface_record::ParticleSurfaceRecord;

/// How an effect's sprite draws: its equation, none for one drawing no colour, whether it distorts, and its record.
pub struct ParticleSurface {
  pub blend: Option<ParticleBlend>,
  pub is_distorting: bool,
  pub record: ParticleSurfaceRecord,
}
