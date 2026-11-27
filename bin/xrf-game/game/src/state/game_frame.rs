use xrf_game_loop::LoopBlend;
use xrf_game_movement::MovementCamera;

/// What presentation and the control endpoint read of the game's state after a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameFrame {
  pub camera: MovementCamera,
  /// How many objects the game holds.
  pub objects: u32,
}

impl LoopBlend for GameFrame {
  /// The camera blends; the count is the next frame's once past this one.
  fn blend(&self, next: &Self, alpha: f32) -> Self {
    Self {
      camera: self.camera.blend(&next.camera, alpha),
      objects: if alpha > 0.0 { next.objects } else { self.objects },
    }
  }
}
