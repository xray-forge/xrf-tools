use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_material::XrayTextureScope;
use xrf_vfs::XraySearch;

use crate::scene::SelectedScene;
use crate::visuals::{SpawnVisualReader, SpawnVisuals};

/// The change event name for stale spawn visuals.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_SPAWN_VISUALS_PART: &str = "sceneSpawnVisuals";

/// Each visual a scene's spawn points are drawn as, read from its game data on first use and kept, or why it cannot be.
/// Packed once per scene and shared by the drawn models and object location, read through the `SpawnVisualReader`
/// [`Self::with_reader`] hands out, as a level's spawned visuals are.
pub(crate) struct SceneSpawnVisualsPart;

impl SessionPart for SceneSpawnVisualsPart {
  type Value = Arc<SpawnVisuals>;

  const NAME: &'static str = SCENE_SPAWN_VISUALS_PART;

  // Built empty, so its build never reaches Rayon; each visual is read later into its own pool-aware cell.
  const IS_POOLED: bool = false;
}

impl SceneSpawnVisualsPart {
  /// Runs `read` with a reader of the open scene's spawn visuals through `game`, a search of its game data, textures
  /// resolved in the shared scope as scene objects' are, and everything it reads recorded as this part's inputs.
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
