use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::level::SelectedLevel;
use crate::render::LevelRenderSource;

/// The source each level session is drawn from, while anything draws it: every viewport showing a snapshot shares
/// one, and a commit moving the session moves its scene onto the next. A clone holds the same sources.
#[derive(Clone, Default)]
pub struct ShownLevels {
  sources: Arc<Mutex<HashMap<SessionId, Weak<LevelRenderSource>>>>,
}

impl ShownLevels {
  /// The source drawing `level`: the one drawing it already, else one following the session's source onto it, keeping
  /// the textures that one resolved, else a new one. A new source may resolve the level's surfaces, so it is built
  /// outside the lock, which a commit's follow waits on; where another show placed one for the same snapshot meanwhile,
  /// that one is answered, so viewports still share one source.
  pub fn get_source(
    &self,
    level: Arc<SessionSnapshot<SelectedLevel>>,
    assets: &AssetMountState,
  ) -> Arc<LevelRenderSource> {
    let shown: Option<Arc<LevelRenderSource>> = self.find_source(level.session_id);

    if let Some(shown) = &shown
      && Arc::ptr_eq(shown.get_level(), &level)
    {
      return Arc::clone(shown);
    }

    let built: Arc<LevelRenderSource> = Arc::new(match shown {
      Some(shown) => shown.follow(level),
      None => LevelRenderSource::new(level, AssetMountState::clone(assets)),
    });
    let mut sources: MutexGuard<'_, HashMap<SessionId, Weak<LevelRenderSource>>> = self.lock();

    if let Some(placed) = sources
      .get(&built.get_level().session_id)
      .and_then(Weak::upgrade)
      .filter(|placed| Arc::ptr_eq(placed.get_level(), built.get_level()))
    {
      return placed;
    }

    sources.retain(|_, source| source.strong_count() > 0);
    sources.insert(built.get_level().session_id, Arc::downgrade(&built));

    built
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

  // Entries change whole, so one a panic left behind is still whole.
  fn lock(&self) -> MutexGuard<'_, HashMap<SessionId, Weak<LevelRenderSource>>> {
    self.sources.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
