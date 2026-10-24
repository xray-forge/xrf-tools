use serde::Serialize;
use xrf_spawn::{SpawnALifeSpawnsChunk, SpawnArtefactSpawnsChunk, SpawnGraphsChunk, SpawnPatrolsChunk};

/// One large chunk of an opened spawn, tagged by its `kind` as [`crate::SpawnChunkKind`] names it.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SpawnChunk {
  AlifeSpawn(SpawnALifeSpawnsChunk),
  ArtefactSpawn(SpawnArtefactSpawnsChunk),
  Patrols(SpawnPatrolsChunk),
  Graphs(SpawnGraphsChunk),
}
