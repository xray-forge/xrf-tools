use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;

use crate::scene::{SceneHelpers, SceneSpawnModelsPart, SelectedScene};

/// What a change event calls a scene's helpers when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_HELPERS_PART: &str = "sceneHelpers";

/// The scene's helpers, built the first time anything asks from the parts its open kept and which spawn points stand
/// as models: it reads no file of its own, and records the game data's lookups those spawn points were found by, so a
/// model come or gone moves it with them. The renderer draws them and the page frames and counts by them.
pub(crate) struct SceneHelpersPart;

impl SessionPart for SceneHelpersPart {
  type Value = Arc<SceneHelpers>;

  const NAME: &'static str = SCENE_HELPERS_PART;
}

impl SceneHelpersPart {
  /// The open scene's helpers, built the first time anything asks and kept, the spawn points' models looked for
  /// through `game`, a search of the scene's game data.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>, game: &XraySearch) -> Arc<SceneHelpers> {
    current.parts.get_or_record::<Self>(game, |game| {
      Arc::new(SceneHelpers::read(current, &SceneSpawnModelsPart::get(current, game)))
    })
  }
}
