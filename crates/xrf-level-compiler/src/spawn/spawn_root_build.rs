use xrf_spawn::SpawnFile;

use crate::spawn::spawn_skipped_connection::SpawnSkippedConnection;

/// A root spawn as the build gives it, and the level connections it left out.
pub struct SpawnRootBuild {
  pub spawn: SpawnFile,
  pub skipped: Vec<SpawnSkippedConnection>,
}
