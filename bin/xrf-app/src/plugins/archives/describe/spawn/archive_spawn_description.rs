use serde::Serialize;
use xrf_db::SpawnHeaderChunk;
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_entry_reader::ArchiveEntryReader;
use crate::plugins::archives::describe::spawn::archive_spawn_section::ArchiveSpawnSection;

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
  /// The spawn set an entry holds, or `None` for a `.spawn` that is not one.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not a chunked container at all.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Option<Self>> {
    let mut reader: ArchiveEntryReader = source.open_entry(name)?;
    let size: u64 = reader.size();

    let Ok(sections) = reader.read_sections() else {
      return Ok(None);
    };

    let Some(header) = reader
      .read_chunk::<SpawnHeaderChunk>(SpawnHeaderChunk::CHUNK_ID)
      .ok()
      .flatten()
    else {
      return Ok(None);
    };

    Ok(Some(Self {
      version: header.version,
      guid: header.guid.to_string(),
      graph_guid: header.graph_guid.to_string(),
      objects: header.objects_count,
      levels: header.levels_count,
      sections: ArchiveSpawnSection::of_all(&sections),
      size,
    }))
  }
}
