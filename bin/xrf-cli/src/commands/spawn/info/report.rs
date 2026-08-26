use serde::Serialize;
use xrf_db::SpawnFile;

/// The level graph a spawn file carries, which is a census of its own.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnGraphReport {
  edges: u32,
  points: u32,
  version: u8,
  vertices: u16,
}

/// What `spawn info` read out of a spawn file.
///
/// The counts and the identity, which is what the command already tells a human. Nothing here is
/// derived: every field is read straight off the file, so the report says what the file says.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnInfoReport {
  artefact_spawn_points: usize,
  graph: SpawnGraphReport,
  guid: String,
  levels: u32,
  objects: u32,
  patrols: usize,
  version: u32,
}

impl SpawnInfoReport {
  pub fn new(file: &SpawnFile) -> Self {
    Self {
      artefact_spawn_points: file.artefact_spawn.nodes.len(),
      graph: SpawnGraphReport {
        edges: file.graphs.header.edges_count,
        points: file.graphs.header.points_count,
        version: file.graphs.header.version,
        vertices: file.graphs.header.vertices_count,
      },
      guid: file.header.guid.to_string(),
      levels: file.header.levels_count,
      objects: file.header.objects_count,
      patrols: file.patrols.patrols.len(),
      version: file.header.version,
    }
  }
}
