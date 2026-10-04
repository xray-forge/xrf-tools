use crate::simulation::particle_collider::ParticleCollider;
use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::particle_library::ParticleLibrary;

/// What an update reads from outside its effects: the library children come from, the engine's rules and the level.
#[derive(Clone, Copy)]
pub struct ParticleUpdateContext<'a> {
  pub library: &'a ParticleLibrary,
  pub rules: &'a ParticleEngineRules,
  /// The surfaces colliding effects meet; none collides with nothing.
  pub collider: Option<&'a dyn ParticleCollider>,
}
