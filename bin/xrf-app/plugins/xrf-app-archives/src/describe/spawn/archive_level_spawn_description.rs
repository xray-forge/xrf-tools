use serde::Serialize;
use xrf_spawn::LevelSpawnFile;

use crate::describe::archive_bounds::ArchiveBounds;
use crate::describe::archive_entry_reader::ArchiveEntryReader;
use crate::describe::spawn::archive_level_spawn_section::ArchiveLevelSpawnSection;

/// Everything the viewer says about the objects a level spawns.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelSpawnDescription {
  pub objects: usize,
  /// What is spawned, grouped by the section each object is built from.
  pub sections: Vec<ArchiveLevelSpawnSection>,
  /// How much of the level the objects stand in, absent for a list holding none.
  pub bounds: Option<ArchiveBounds>,
  pub size: u64,
}

impl ArchiveLevelSpawnDescription {
  /// The spawn list an opened entry holds, or `None` for a `.spawn` that is not one; every object is read, so the
  /// caller weighs the entry against the ceiling first.
  pub fn of_entry(entry: &mut ArchiveEntryReader) -> Option<Self> {
    let file: LevelSpawnFile = entry.read_container::<LevelSpawnFile>().ok()?;

    Some(Self {
      objects: file.objects.len(),
      sections: ArchiveLevelSpawnSection::of_all(&file),
      bounds: ArchiveBounds::of_points(file.objects.iter().map(|object| &object.position)),
      size: entry.size(),
    })
  }
}
