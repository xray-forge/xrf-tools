use std::sync::Arc;

use rayon::ThreadPool;
use xrf_app_core::session::{SessionScopes, Sessions};

use crate::opened_level::OpenedLevel;
use crate::render_source::LevelRenderSource;
use crate::state::selection::selected_level::SelectedLevel;

/// The levels open, each held by the tool that opened it.
pub struct LevelState {
  pub levels: Sessions<SelectedLevel>,
}

impl LevelState {
  /// The state of levels read again on `pool` once a commit moved one of their own files: a read takes seconds, which no
  /// save waits for; whatever scope `scopes` releases closes the levels it held. A change renews a level's scene where
  /// it lets what the scene is built from go.
  pub fn new(pool: Arc<ThreadPool>, scopes: &SessionScopes) -> Self {
    Self {
      levels: Sessions::new("level", scopes)
        .with_deferred_derivation(pool, OpenedLevel::derive)
        .with_view_parts(LevelRenderSource::VIEW_PARTS),
    }
  }
}
