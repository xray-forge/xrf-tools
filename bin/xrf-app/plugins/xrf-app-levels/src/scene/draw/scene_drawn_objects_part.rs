use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;

use crate::scene::{SceneDrawnObjects, SelectedScene};

/// The change event name for stale drawn scene objects.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_DRAWN_OBJECTS_PART: &str = "sceneDrawnObjects";

/// The scene objects a scene draws, read on first use: the one read both the page's frame and the renderer's picture
/// come from. It records every `.object` it looked for, so one added or removed in the objects roots moves this part
/// and the parts read from it, not the scene.
pub(crate) struct SceneDrawnObjectsPart;

impl SessionPart for SceneDrawnObjectsPart {
  type Value = Arc<SceneDrawnObjects>;

  const NAME: &'static str = SCENE_DRAWN_OBJECTS_PART;
}

impl SceneDrawnObjectsPart {
  /// The open scene's drawn objects, read on first use and kept, their library objects looked for through `objects`, a
  /// search of the scene's objects roots.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>, objects: &XraySearch) -> Arc<SceneDrawnObjects> {
    current
      .parts
      .get_or_record::<Self>(objects, |objects| Arc::new(SceneDrawnObjects::read(current, objects)))
  }
}
