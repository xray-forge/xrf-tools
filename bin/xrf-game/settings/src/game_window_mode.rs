use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GameWindowMode {
  /// A framed window the player can move and resize, at the settings' size.
  Windowed,
  /// Borderless over the whole display, at the display's own mode.
  #[default]
  Fullscreen,
}
