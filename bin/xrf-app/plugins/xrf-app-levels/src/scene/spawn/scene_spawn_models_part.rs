use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;

use crate::scene::{SceneSpawnModels, SceneSpawnVisualsPart, SelectedScene};

/// The change event name for stale spawn models.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_SPAWN_MODELS_PART: &str = "sceneSpawnModels";

/// Which of a scene's spawn points are drawn as their entities' models, read on first use by looking for each packet's
/// visual in the game data without reading it. It records every lookup, so a model added or removed in the game data
/// moves this part and not the scene.
pub(crate) struct SceneSpawnModelsPart;

impl SessionPart for SceneSpawnModelsPart {
  type Value = Arc<SceneSpawnModels>;

  const NAME: &'static str = SCENE_SPAWN_MODELS_PART;
}

impl SceneSpawnModelsPart {
  /// The open scene's spawn models, read on first use and kept, their visuals looked for through `game`, a search of
  /// the scene's game data.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>, game: &XraySearch) -> Arc<SceneSpawnModels> {
    current.parts.get_or_record::<Self>(game, |game| {
      Arc::new(SceneSpawnVisualsPart::with_reader(current, game, |reader| {
        SceneSpawnModels::read(current, reader)
      }))
    })
  }
}
