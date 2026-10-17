use serde::Serialize;
use xrf_spawn::SpawnFile;

use crate::commands::spawn::info::spawn_graph_report::SpawnGraphReport;

/// What `spawn info` read out of a spawn file.
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
      graph: SpawnGraphReport::new(&file.graphs),
      guid: file.header.guid.to_string(),
      levels: file.header.levels_count,
      objects: file.header.objects_count,
      patrols: file.patrols.patrols.len(),
      version: file.header.version,
    }
  }
}
