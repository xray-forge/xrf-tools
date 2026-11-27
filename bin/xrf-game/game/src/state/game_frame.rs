use xrf_game_loop::LoopBlend;
use xrf_game_movement::{MovementCamera, MovementFooting};

use crate::time::game_clock::GameClock;

/// What presentation and the control endpoint read of the game's state after a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameFrame {
  pub camera: MovementCamera,
  /// What the walking actor stands on, which its footsteps play by; none while flying.
  pub footing: Option<MovementFooting>,
  /// Game seconds since midnight.
  pub time_of_day: f32,
  /// Game seconds a simulated second.
  pub time_factor: f32,
  /// How many objects the game holds, and how many of them are online.
  pub objects: u32,
  pub online: u32,
  /// How many squads the game holds, and how many members they have.
  pub squads: u32,
  pub members: u32,
}

impl LoopBlend for GameFrame {
  /// The camera blends and the day runs forward across midnight; everything else is the next frame's once past this
  /// one.
  fn blend(&self, next: &Self, alpha: f32) -> Self {
    let latest: &Self = if alpha > 0.0 { next } else { self };
    let ahead: f32 = (next.time_of_day - self.time_of_day).rem_euclid(GameClock::DAY);

    Self {
      camera: self.camera.blend(&next.camera, alpha),
      time_of_day: (self.time_of_day + ahead * alpha).rem_euclid(GameClock::DAY),
      ..*latest
    }
  }
}
