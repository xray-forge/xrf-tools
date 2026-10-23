use std::sync::Arc;

use rayon::ThreadPool;

use crate::core::session::Session;
use crate::plugins::levels::opened_level::OpenedLevel;
use crate::plugins::levels::state::selection::selected_level::SelectedLevel;

/// Ownership for the level a viewer has open.
pub struct LevelState {
  pub selected: Session<SelectedLevel>,
}

impl LevelState {
  /// The state of a level read again on `pool` once a commit moved one of its own files: a read takes seconds, which no
  /// save waits for.
  pub fn new(pool: Arc<ThreadPool>) -> Self {
    Self {
      selected: Session::new("level").with_deferred_derivation(pool, OpenedLevel::derive),
    }
  }
}
