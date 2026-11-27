use serde::{Deserialize, Serialize};

/// One step of input an agent plays into the game, as the window would deliver it, named by its `type`. Keys are named
/// as SDL names scancodes: `W`, `Grave` (the console key), `Return`, `Escape`, `Space`, `Left Shift`, `Up`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameInput {
  /// Presses a key and lets it go at once.
  Press {
    key: String,
  },
  /// Holds a key down until an `up` for it.
  Down {
    key: String,
  },
  Up {
    key: String,
  },
  /// Types text, as the window gives it while it accepts text.
  Text {
    text: String,
  },
  /// Moves the mouse by `dx` and `dy`, turning the camera while the cursor is grabbed; `x` and `y` place the cursor,
  /// in the window's coordinates, where it is free.
  Move {
    dx: f32,
    dy: f32,
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
  },
  /// Clicks `button` (`left`, `right` or `middle`, `left` unless given) at `x` and `y` in the window's coordinates.
  Click {
    x: f32,
    y: f32,
    #[serde(default)]
    button: Option<String>,
  },
  /// Turns the mouse wheel by `dy` lines, up for more.
  Wheel {
    dy: f32,
  },
  /// Waits before the steps after it.
  Wait {
    ms: u64,
  },
}
