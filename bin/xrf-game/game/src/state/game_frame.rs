use xrf_game_loop::LoopBlend;
use xrf_game_movement::MovementCamera;

/// What presentation reads of the game's state after a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameFrame {
  pub camera: MovementCamera,
}

impl LoopBlend for GameFrame {
  fn blend(&self, next: &Self, alpha: f32) -> Self {
    Self {
      camera: self.camera.blend(&next.camera, alpha),
    }
  }
}
