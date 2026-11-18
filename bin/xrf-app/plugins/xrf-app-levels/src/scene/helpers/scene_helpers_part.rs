use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;

use crate::scene::{SceneHelpers, SceneSpawnModelsPart, SelectedScene};

/// The change event name for stale scene helpers.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_HELPERS_PART: &str = "sceneHelpers";

/// The scene's helpers, built on first use from the parts the open kept and the spawn models. It reads no file of its
/// own but records the game data lookups that found the spawn models, so a model added or removed moves it with them.
pub(crate) struct SceneHelpersPart;

impl SessionPart for SceneHelpersPart {
  type Value = Arc<SceneHelpers>;

  const NAME: &'static str = SCENE_HELPERS_PART;
}

impl SceneHelpersPart {
  /// The open scene's helpers, built on first use and kept, the spawn models looked for through `game`, a search of the
  /// scene's game data.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>, game: &XraySearch) -> Arc<SceneHelpers> {
    current.parts.get_or_record::<Self>(game, |game| {
      Arc::new(SceneHelpers::read(current, &SceneSpawnModelsPart::get(current, game)))
    })
  }
}
