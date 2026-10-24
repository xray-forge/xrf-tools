use serde::{Deserialize, Serialize};

/// Which of an opened spawn's large chunks to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum SpawnChunkKind {
  AlifeSpawn,
  ArtefactSpawn,
  Patrols,
  Graphs,
}
