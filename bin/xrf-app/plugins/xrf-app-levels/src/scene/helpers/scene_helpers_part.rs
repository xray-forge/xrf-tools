use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};

use crate::scene::{SceneHelpers, SelectedScene};

/// What a change event calls a scene's helpers when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_HELPERS_PART: &str = "sceneHelpers";

/// The scene's helpers, built the first time anything asks from the parts its open kept: it reads no file of its own,
/// so it goes with the scene. The renderer draws them and the page frames and counts by them.
pub(crate) struct SceneHelpersPart;

impl SessionPart for SceneHelpersPart {
  type Value = Arc<SceneHelpers>;

  const NAME: &'static str = SCENE_HELPERS_PART;
}

impl SceneHelpersPart {
  /// The open scene's helpers, built the first time anything asks and kept.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>) -> Arc<SceneHelpers> {
    current
      .parts
      .get_or_init::<Self>(|| Arc::new(SceneHelpers::read(current)))
  }
}
