use serde::Serialize;

use crate::commands::spawn::info::spawn_info_file::SpawnInfoFile;
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
  pub(crate) fn new(file: &SpawnInfoFile) -> Self {
    match file {
      SpawnInfoFile::Root(file) => Self::Root(SpawnRootReport::new(file)),
      SpawnInfoFile::Level(file) => Self::Level(SpawnLevelReport::new(file)),
    }
  }
}
