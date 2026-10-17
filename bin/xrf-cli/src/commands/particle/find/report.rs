use serde::Serialize;
use xrf_particles::ParticleActionType;

use crate::commands::particle::find::particle_find_match::ParticleFindMatch;

/// What `particle find` answers: the queries, echoed back, and every effect or group matching one, effects first, each
/// in the library's order.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleFindReport {
  pub textures: Vec<String>,
  pub shaders: Vec<String>,
  pub actions: Vec<ParticleActionType>,
  pub effects: Vec<String>,
  pub matches: Vec<ParticleFindMatch>,
}
