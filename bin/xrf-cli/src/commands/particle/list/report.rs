use serde::Serialize;

use crate::commands::particle::list::particle_list_entry::ParticleListEntry;

/// What `particle list` answers: the effects and groups of a library a filter keeps, effects first, each in the
/// library's order.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleListReport {
  entries: Vec<ParticleListEntry>,
}

impl ParticleListReport {
  pub fn new(entries: Vec<ParticleListEntry>) -> Self {
    Self { entries }
  }

  pub fn entries(&self) -> &[ParticleListEntry] {
    &self.entries
  }
}
