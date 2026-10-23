use std::sync::Arc;

use rayon::ThreadPool;

use crate::core::session::{SessionScopes, Sessions};
use crate::plugins::levels::opened_level::OpenedLevel;
use crate::plugins::levels::state::selection::selected_level::SelectedLevel;

/// The levels open, each held by the tool that opened it.
pub struct LevelState {
  pub levels: Sessions<SelectedLevel>,
}

impl LevelState {
  /// The state of levels read again on `pool` once a commit moved one of their own files: a read takes seconds, which no
  /// save waits for; whatever scope `scopes` releases closes the levels it held.
  pub fn new(pool: Arc<ThreadPool>, scopes: &SessionScopes) -> Self {
    Self {
      levels: Sessions::new("level", scopes).with_deferred_derivation(pool, OpenedLevel::derive),
    }
  }
}
