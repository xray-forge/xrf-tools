use crate::core::session::Session;
use crate::plugins::levels::state::selection::selected_level::SelectedLevel;

/// Ownership for the level a viewer has open.
pub struct LevelState {
  pub selected: Session<SelectedLevel>,
}

impl LevelState {
  pub fn new() -> Self {
    Self {
      selected: Session::new("level"),
    }
  }
}
