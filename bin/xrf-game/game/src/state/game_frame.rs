use xrf_game_movement::MovementCamera;
use xrf_math::glam::Vec3;

use crate::state::game_pose::GamePose;
use crate::time::game_clock::GameClock;

/// What presentation samples of the game's state after a tick, shared rather than copied: the player's camera and how
/// fast it moves, which presentation carries it forward by, the game's day, and where every online object stands, in id
/// order so two frames' tables blend in one walk. What happens once is an event (`GameEvent`); what a tool asks of the
/// state is a query between ticks, never a field here.
#[derive(Clone, Debug, PartialEq)]
pub struct GameFrame {
  pub camera: MovementCamera,
  /// Metres a second the camera moved at over the tick.
  pub velocity: Vec3,
  /// Game seconds since midnight.
  pub time_of_day: f32,
  /// Game seconds a simulated second.
  pub time_factor: f32,
  /// Every online object, by id.
  pub poses: Vec<GamePose>,
}

impl GameFrame {
  /// The time of day `alpha` of the way from this frame to `next`, running forward across midnight.
  pub fn blend_time_of_day(&self, next: &Self, alpha: f32) -> f32 {
    let ahead: f32 = (next.time_of_day - self.time_of_day).rem_euclid(GameClock::DAY);

    (self.time_of_day + ahead * alpha).rem_euclid(GameClock::DAY)
  }
}
