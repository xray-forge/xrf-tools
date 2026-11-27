use serde::{Deserialize, Serialize};

use crate::game_window_mode::GameWindowMode;

/// How the game's window opens.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameWindowSettings {
  pub mode: GameWindowMode,
  /// Its size while windowed, in the display's logical units.
  pub width: u32,
  pub height: u32,
}

impl Default for GameWindowSettings {
  fn default() -> Self {
    Self {
      mode: GameWindowMode::Fullscreen,
      width: 1920,
      height: 1080,
    }
  }
}
