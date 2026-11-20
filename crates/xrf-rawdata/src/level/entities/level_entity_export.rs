use xrf_spawn::LevelSpawnFile;

/// The spawn lists a scene's entities export to, each `None` where no entity goes there, as the build then writes none.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelEntityExport {
  pub spawn: Option<LevelSpawnFile>,
  pub respawn: Option<LevelSpawnFile>,
}
