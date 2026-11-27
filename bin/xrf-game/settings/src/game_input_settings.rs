use serde::{Deserialize, Serialize};

use crate::game_binding_settings::GameBindingSettings;

/// How the player's input reads: how far the mouse turns the look, and the keys bound in place of the tree's own, each
/// action's main key (`bind`) and its second (`bind_sec`), as X-Ray binds them.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInputSettings {
  /// Radians a count of the mouse turns the look by, in X-Ray's `mouse_sens` place.
  pub sensitivity: f32,
  pub bindings: GameBindingSettings,
  pub bindings_secondary: GameBindingSettings,
}

impl Default for GameInputSettings {
  fn default() -> Self {
    Self {
      sensitivity: Self::SENSITIVITY,
      bindings: GameBindingSettings::default(),
      bindings_secondary: GameBindingSettings::default(),
    }
  }
}

impl GameInputSettings {
  /// The sensitivity the game starts with: 0.0025 rad a count.
  pub const SENSITIVITY: f32 = 0.0025;

  /// The bounds the sensitivity is held to: some turn, and at most a tenth of a radian a count.
  const SENSITIVITY_BOUNDS: [f32; 2] = [0.000_01, 0.1];

  /// Holds the sensitivity to its bounds.
  pub fn hold_to_bounds(&mut self) {
    let [lowest, highest] = Self::SENSITIVITY_BOUNDS;

    self.sensitivity = self.sensitivity.clamp(lowest, highest);
  }
}
