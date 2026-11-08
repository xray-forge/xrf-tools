use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParticlesFirstgenChunk {}

impl ParticlesFirstgenChunk {
  pub(crate) const CHUNK_ID: u32 = 2;
}
