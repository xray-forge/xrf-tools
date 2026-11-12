use serde::Serialize;
use xrf_spawn::AnySpawnFile;

use crate::commands::spawn::info::spawn_level_report::SpawnLevelReport;
use crate::commands::spawn::info::spawn_root_report::SpawnRootReport;

/// What `spawn info` read out of a spawn file, beside the `kind` of `.spawn` it was.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum SpawnInfoReport {
  Root(SpawnRootReport),
  Level(SpawnLevelReport),
}

impl SpawnInfoReport {
  pub(crate) fn new(file: &AnySpawnFile) -> Self {
    match file {
      AnySpawnFile::Root(file) => Self::Root(SpawnRootReport::new(file)),
      AnySpawnFile::Level(file) => Self::Level(SpawnLevelReport::new(file)),
    }
  }
}
