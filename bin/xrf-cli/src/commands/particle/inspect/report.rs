use serde::Serialize;
use xrf_particles::{ParticleEffect, ParticleGroup};

/// What `particle inspect` answers: the effect or group of the name, whole, as the library holds it; both where an
/// effect and a group share it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleInspectReport<'a> {
  pub effect: Option<&'a ParticleEffect>,
  pub group: Option<&'a ParticleGroup>,
}
