use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;

use crate::scene::{SceneDrawnObjects, SelectedScene};

/// What a change event calls the scene objects a scene draws when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_DRAWN_OBJECTS_PART: &str = "sceneDrawnObjects";

/// The scene objects a scene draws, read the first time anything asks: the one read the page's frame and the
/// renderer's picture both come from. It records every `.object` it looked for, so one come or gone in the objects
/// roots moves this part, and every part read from it, and not the scene.
pub(crate) struct SceneDrawnObjectsPart;

impl SessionPart for SceneDrawnObjectsPart {
  type Value = Arc<SceneDrawnObjects>;

  const NAME: &'static str = SCENE_DRAWN_OBJECTS_PART;
}

impl SceneDrawnObjectsPart {
  /// The open scene's drawn objects, read the first time anything asks and kept, their library objects looked for
  /// through `objects`, a search of the scene's objects roots.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>, objects: &XraySearch) -> Arc<SceneDrawnObjects> {
    current
      .parts
      .get_or_record::<Self>(objects, |objects| Arc::new(SceneDrawnObjects::read(current, objects)))
  }
}
