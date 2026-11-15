use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_material::XrayTextureScope;
use xrf_vfs::XraySearch;

use crate::level::SelectedLevel;
use crate::visuals::{SpawnVisualReader, SpawnVisuals};

/// What a change event calls a level's spawned objects' visuals when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SPAWN_VISUALS_PART: &str = "spawnVisuals";

/// Each visual a level's spawned objects stand as, read the first time anything asks and kept, or why it cannot be:
/// packed once a level and shared by whatever reads them, the models drawn and the lights placed on their bones. Read
/// through a `SpawnVisualReader` [`Self::with_reader`] hands out.
pub(crate) struct LevelSpawnVisualsPart;

impl SessionPart for LevelSpawnVisualsPart {
  type Value = Arc<SpawnVisuals>;

  const NAME: &'static str = LEVEL_SPAWN_VISUALS_PART;

  // Built empty, so its build never reaches Rayon; its visuals are each read later into a pool-aware cell of their own.
  const IS_POOLED: bool = false;
}

impl LevelSpawnVisualsPart {
  /// Runs `read` with a reader of the open level's spawned visuals, looking beside the level first and resolving
  /// textures in its scope, whatever it reads recorded as this part read it.
  pub(crate) fn with_reader<T>(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    read: impl FnOnce(&SpawnVisualReader) -> T,
  ) -> T {
    let visuals: Arc<SpawnVisuals> = current.parts.get_or_init::<Self>(|| Arc::new(SpawnVisuals::new()));
    let scope: XrayTextureScope = current.source.get_texture_scope();

    current.parts.record::<Self, T>(search, |search| {
      read(&SpawnVisualReader::new(visuals, search.clone(), scope).beside(current.source.get_logical_directory()))
    })
  }
}
