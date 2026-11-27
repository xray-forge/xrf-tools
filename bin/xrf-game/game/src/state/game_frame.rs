use xrf_game_loop::LoopBlend;
use xrf_game_movement::MovementCamera;

/// What presentation and the control endpoint read of the game's state after a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameFrame {
  pub camera: MovementCamera,
  /// How many objects the game holds, and how many of them are online.
  pub objects: u32,
  pub online: u32,
  /// How many squads the game holds, and how many members they have.
  pub squads: u32,
  pub members: u32,
}

impl LoopBlend for GameFrame {
  /// The camera blends; the counts are the next frame's once past this one.
  fn blend(&self, next: &Self, alpha: f32) -> Self {
    let counts: &Self = if alpha > 0.0 { next } else { self };

    Self {
      camera: self.camera.blend(&next.camera, alpha),
      objects: counts.objects,
      online: counts.online,
      squads: counts.squads,
      members: counts.members,
    }
  }
}
