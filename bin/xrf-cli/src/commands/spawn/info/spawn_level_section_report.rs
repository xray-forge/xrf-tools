use serde::Serialize;
use xrf_spawn::LevelSpawnFile;

/// One config section a level spawns objects from, and how many.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpawnLevelSectionReport {
  name: String,
  objects: usize,
}

impl SpawnLevelSectionReport {
  /// Every section the list spawns from, by name.
  pub(crate) fn list(file: &LevelSpawnFile) -> Vec<Self> {
    file
      .get_sections()
      .into_iter()
      .map(|(name, objects)| Self {
        name: name.to_owned(),
        objects,
      })
      .collect()
  }
}
