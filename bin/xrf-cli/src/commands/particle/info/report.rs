use serde::Serialize;
use xrf_db::ParticlesFile;

/// What `particle info` read out of a particle library.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleInfoReport {
  effects: usize,
  groups: usize,
  version: u16,
}

impl ParticleInfoReport {
  pub fn new(file: &ParticlesFile) -> Self {
    Self {
      effects: file.effects.effects.len(),
      groups: file.groups.groups.len(),
      version: file.header.version,
    }
  }
}
