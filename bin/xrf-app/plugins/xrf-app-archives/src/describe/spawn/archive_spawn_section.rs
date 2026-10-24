use serde::Serialize;
use xrf_spawn::{
  SpawnALifeSpawnsChunk, SpawnArtefactSpawnsChunk, SpawnGraphsChunk, SpawnHeaderChunk, SpawnPatrolsChunk,
};

/// One top-level section of a spawn set, by id and weight.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSpawnSection {
  pub id: u32,
  /// What the section holds, where the format names it.
  pub label: Option<String>,
  pub size: u64,
}

impl ArchiveSpawnSection {
  /// Every top-level section, read off the chunk headers alone.
  pub fn of_all(sections: &[(u32, u64)]) -> Vec<Self> {
    sections
      .iter()
      .map(|(id, size)| Self {
        id: *id,
        label: Self::to_label(*id).map(ToOwned::to_owned),
        size: *size,
      })
      .collect()
  }

  /// The name the format gives a section id.
  const fn to_label(id: u32) -> Option<&'static str> {
    match id {
      SpawnHeaderChunk::CHUNK_ID => Some("Header"),
      SpawnALifeSpawnsChunk::CHUNK_ID => Some("ALife objects"),
      SpawnArtefactSpawnsChunk::CHUNK_ID => Some("Artefact spawns"),
      SpawnPatrolsChunk::CHUNK_ID => Some("Patrols"),
      SpawnGraphsChunk::CHUNK_ID => Some("Game graph"),
      _ => None,
    }
  }
}
