/// Where a scene is heard from, in the game's left-handed space: `x` right, `y` up, `z` ahead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioListener {
  pub position: [f32; 3],
  /// Unit vectors it looks along and holds up.
  pub forward: [f32; 3],
  pub up: [f32; 3],
  /// Metres a second.
  pub velocity: [f32; 3],
}

impl Default for AudioListener {
  fn default() -> Self {
    Self {
      position: [0.0; 3],
      forward: [0.0, 0.0, 1.0],
      up: [0.0, 1.0, 0.0],
      velocity: [0.0; 3],
    }
  }
}
