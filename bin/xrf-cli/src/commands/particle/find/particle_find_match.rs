use serde::Serialize;

use crate::commands::particle::find::particle_find_reason::ParticleFindReason;
use crate::commands::particle::list::particle_list_entry::ParticleListEntry;

/// An effect or group matching a query, as `particle list` describes it, with every way it matched.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleFindMatch {
  #[serde(flatten)]
  pub entry: ParticleListEntry,
  pub reasons: Vec<ParticleFindReason>,
}
