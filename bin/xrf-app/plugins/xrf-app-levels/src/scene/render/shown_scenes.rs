use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::scene::{SceneRenderSource, SelectedScene};

/// The source each scene session is drawn from while anything draws it. Every viewport showing a snapshot shares one,
/// so what one compiled the others draw. A clone holds the same sources.
#[derive(Clone, Default)]
pub struct ShownScenes {
  sources: Arc<Mutex<HashMap<SessionId, Weak<SceneRenderSource>>>>,
}

impl ShownScenes {
  /// The source already drawing `scene`, else a new one that reads nothing until the renderer asks.
  pub fn get_source(
    &self,
    scene: Arc<SessionSnapshot<SelectedScene>>,
    assets: &AssetMountState,
  ) -> Arc<SceneRenderSource> {
    let mut sources: MutexGuard<'_, HashMap<SessionId, Weak<SceneRenderSource>>> = self.lock();

    if let Some(shown) = sources
      .get(&scene.session_id)
      .and_then(Weak::upgrade)
      .filter(|shown| Self::is_drawing(shown, &scene))
    {
      return shown;
    }

    let source: Arc<SceneRenderSource> = Arc::new(SceneRenderSource::new(scene, AssetMountState::clone(assets)));

    sources.retain(|_, source| source.strong_count() > 0);
    sources.insert(source.get_scene().session_id, Arc::downgrade(&source));

    source
  }

  /// The source a session is drawn from now, none where nothing draws it.
  #[cfg(test)]
  pub fn find_source(&self, session_id: SessionId) -> Option<Arc<SceneRenderSource>> {
    self.lock().get(&session_id).and_then(Weak::upgrade)
  }

  /// Whether `source` draws `scene`, or a snapshot of its session published after it.
  fn is_drawing(source: &SceneRenderSource, scene: &SessionSnapshot<SelectedScene>) -> bool {
    source.get_scene().revision >= scene.revision
  }

  // Entries change whole, so one a panic left behind is still whole.
  fn lock(&self) -> MutexGuard<'_, HashMap<SessionId, Weak<SceneRenderSource>>> {
    self.sources.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
