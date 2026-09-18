use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesFirstgenChunk {}

impl ParticlesFirstgenChunk {
  pub const CHUNK_ID: u32 = 2;
}
