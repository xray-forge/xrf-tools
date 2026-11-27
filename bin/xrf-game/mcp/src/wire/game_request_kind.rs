use serde::{Deserialize, Serialize};

use crate::wire::game_input::GameInput;

/// What a request asks of the game, named by its line's `kind`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameRequestKind {
  /// Where the game stands: its level, tick, game time and camera.
  Status,
  /// Stands the camera at `position` looking at `target`, in the engine's space.
  Pose { position: [f32; 3], target: [f32; 3] },
  /// Plays `inputs` into the game in order, after any played before; answered once the last is delivered.
  Input { inputs: Vec<GameInput> },
  /// Reads the window's next frame back and writes it to a PNG file, answered with its path.
  Capture,
  /// Quits the game as its close button does.
  Quit,
}
