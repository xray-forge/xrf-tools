use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_material::XrayTextureScope;
use xrf_vfs::XraySearch;

use crate::scene::SelectedScene;
use crate::visuals::{SpawnVisualReader, SpawnVisuals};

/// What a change event calls the visuals a scene's spawn points stand as when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_SPAWN_VISUALS_PART: &str = "sceneSpawnVisuals";

/// Each visual a scene's spawn points stand as, read from its game data the first time anything asks and kept, or why
/// it cannot be: packed once a scene and shared by whatever reads them, the models drawn and where one is located. Read
/// through a `SpawnVisualReader` [`Self::with_reader`] hands out, as a level's spawned visuals are.
pub(crate) struct SceneSpawnVisualsPart;

impl SessionPart for SceneSpawnVisualsPart {
  type Value = Arc<SpawnVisuals>;

  const NAME: &'static str = SCENE_SPAWN_VISUALS_PART;

  // Built empty, so its build never reaches Rayon; its visuals are each read later into a pool-aware cell of their own.
  const IS_POOLED: bool = false;
}

impl SceneSpawnVisualsPart {
  /// Runs `read` with a reader of the open scene's spawn points' visuals through `game`, a search of its game data,
  /// their textures resolved in the shared scope as its scene objects' are, whatever it reads recorded as this part
  /// read it.
  pub(crate) fn with_reader<T>(
    current: &SessionSnapshot<SelectedScene>,
    game: &XraySearch,
    read: impl FnOnce(&SpawnVisualReader) -> T,
  ) -> T {
    let visuals: Arc<SpawnVisuals> = current.parts.get_or_init::<Self>(|| Arc::new(SpawnVisuals::new()));

    current.parts.record::<Self, T>(game, |game| {
      read(&SpawnVisualReader::new(
        visuals,
        game.clone(),
        XrayTextureScope::shared(),
      ))
    })
  }
}
