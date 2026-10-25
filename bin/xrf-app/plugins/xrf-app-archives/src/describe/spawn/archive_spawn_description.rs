use serde::Serialize;
use xrf_spawn::SpawnHeaderChunk;

use crate::describe::source::ArchiveEntryReader;
use crate::describe::spawn::archive_spawn_section::ArchiveSpawnSection;

/// What a spawn set holds, taken from its header and the weight of its sections.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSpawnDescription {
  pub version: u32,
  /// Identity of this build of the set, which a save game is pinned to.
  pub guid: String,
  /// Identity of the game graph the set was built against.
  pub graph_guid: String,
  pub objects: u32,
  pub levels: u32,
  /// Top-level sections in the order the file frames them, with what each weighs.
  pub sections: Vec<ArchiveSpawnSection>,
  pub size: u64,
}

impl ArchiveSpawnDescription {
  /// The spawn set an opened entry holds, or `None` for a `.spawn` that is not one, read by the headers and the one
  /// chunk that names the set.
  pub fn of_entry(entry: &mut ArchiveEntryReader) -> Option<Self> {
    let size: u64 = entry.size();
    let sections: Vec<(u32, u64)> = entry.read_sections().ok()?;
    let header: SpawnHeaderChunk = entry
      .read_chunk::<SpawnHeaderChunk>(SpawnHeaderChunk::CHUNK_ID)
      .ok()
      .flatten()?;

    Some(Self {
      version: header.version,
      guid: header.guid.to_string(),
      graph_guid: header.graph_guid.to_string(),
      objects: header.objects_count,
      levels: header.levels_count,
      sections: ArchiveSpawnSection::of_all(&sections),
      size,
    })
  }
}
