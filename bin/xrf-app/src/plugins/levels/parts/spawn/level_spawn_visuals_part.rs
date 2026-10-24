use std::sync::Arc;

use crate::core::session::SessionPart;
use crate::plugins::levels::state::LevelSpawnVisuals;

/// What a change event calls a level's spawned objects' visuals when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SPAWN_VISUALS_PART: &str = "spawnVisuals";

/// Each visual a level's spawned objects stand as, read the first time anything asks and kept, or why it cannot be:
/// packed once a level and shared by whatever reads them, the models drawn and the lights placed on their bones. Read
/// through a `SpawnVisualReader`.
pub(crate) struct LevelSpawnVisualsPart;

impl SessionPart for LevelSpawnVisualsPart {
  type Value = Arc<LevelSpawnVisuals>;

  const NAME: &'static str = LEVEL_SPAWN_VISUALS_PART;

  // Built empty, its visuals each read into a cell of its own later, so its build never reaches Rayon.
  const IS_POOLED: bool = false;
}
