use std::collections::HashMap;
use std::sync::Arc;

use xrf_material::XraySurfaceDescriptor;
use xrf_particles::{ParticleCollider, ParticleEngineRules, ParticleLibrary};

/// What a level's particles are made from, as the world read it: the effects and groups, the engine's rules, each
/// effect's surface, and what particles collide with.
pub struct RenderParticleDefinitions {
  pub library: Arc<ParticleLibrary>,
  pub rules: ParticleEngineRules,
  pub surfaces: HashMap<String, XraySurfaceDescriptor>,
  pub collider: Option<Arc<dyn ParticleCollider>>,
}
