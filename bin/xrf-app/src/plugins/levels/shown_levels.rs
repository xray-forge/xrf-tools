use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::plugins::levels::render_source::LevelRenderSource;
use crate::plugins::levels::state::SelectedLevel;

/// The source each level session is drawn from, while anything draws it: every viewport showing a snapshot shares
/// one, and a commit moving the session moves its scene onto the next. A clone holds the same sources.
#[derive(Clone, Default)]
pub struct ShownLevels {
  sources: Arc<Mutex<HashMap<SessionId, Weak<LevelRenderSource>>>>,
}

impl ShownLevels {
  /// The source drawing `level`: the one drawing it already, else one following the session's source onto it, keeping
  /// the textures that one resolved, else a new one.
  pub fn get_source(
    &self,
    level: Arc<SessionSnapshot<SelectedLevel>>,
    assets: &AssetMountState,
  ) -> Arc<LevelRenderSource> {
    let mut sources: MutexGuard<'_, HashMap<SessionId, Weak<LevelRenderSource>>> = self.lock();
    let source: Arc<LevelRenderSource> = match sources.get(&level.session_id).and_then(Weak::upgrade) {
      Some(shown) if Arc::ptr_eq(shown.get_level(), &level) => return shown,
      Some(shown) => Arc::new(shown.follow(level)),
      None => Arc::new(LevelRenderSource::new(level, AssetMountState::clone(assets))),
    };

    sources.retain(|_, source| source.strong_count() > 0);
    sources.insert(source.get_level().session_id, Arc::downgrade(&source));

    source
  }

  /// The source a session is drawn from now, none where nothing draws it.
  pub fn find_source(&self, session_id: SessionId) -> Option<Arc<LevelRenderSource>> {
    self.lock().get(&session_id).and_then(Weak::upgrade)
  }

  /// Takes `source` as the one its session is drawn from from now on.
  pub fn set_source(&self, source: &Arc<LevelRenderSource>) {
    self
      .lock()
      .insert(source.get_level().session_id, Arc::downgrade(source));
  }

  fn lock(&self) -> MutexGuard<'_, HashMap<SessionId, Weak<LevelRenderSource>>> {
    self.sources.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
