use serde::Serialize;
use xrf_spawn::LevelSpawnFile;

use crate::commands::spawn::info::spawn_level_section_report::SpawnLevelSectionReport;

/// What `spawn info` read out of a level's spawn list: the objects counted by the section each is built from, which
/// sum to the list's object count.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpawnLevelReport {
  sections: Vec<SpawnLevelSectionReport>,
}

impl SpawnLevelReport {
  pub(crate) fn new(file: &LevelSpawnFile) -> Self {
    Self {
      sections: SpawnLevelSectionReport::list(file),
    }
  }
}
