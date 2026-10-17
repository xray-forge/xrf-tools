use serde::Serialize;
use xrf_spawn::SpawnGraphsChunk;

/// The level graph a spawn file carries, which is a census of its own.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnGraphReport {
  edges: u32,
  points: u32,
  version: u8,
  vertices: u16,
}

impl SpawnGraphReport {
  pub fn new(graphs: &SpawnGraphsChunk) -> Self {
    Self {
      edges: graphs.header.edges_count,
      points: graphs.header.points_count,
      version: graphs.header.version,
      vertices: graphs.header.vertices_count,
    }
  }
}
