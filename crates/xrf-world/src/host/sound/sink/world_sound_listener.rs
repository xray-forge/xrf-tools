/// Where a scene is heard from, in engine space: the listening viewport's camera, which stands for the actor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldSoundListener {
  pub position: [f32; 3],
  /// Unit vectors the camera looks along and holds up.
  pub forward: [f32; 3],
  pub up: [f32; 3],
  /// Metres a second.
  pub velocity: [f32; 3],
}

impl Default for WorldSoundListener {
  fn default() -> Self {
    Self {
      position: [0.0; 3],
      forward: [0.0, 0.0, 1.0],
      up: [0.0, 1.0, 0.0],
      velocity: [0.0; 3],
    }
  }
}
