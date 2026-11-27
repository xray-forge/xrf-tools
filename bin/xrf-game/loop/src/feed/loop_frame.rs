use crate::clock::game_clock::GameClock;
use crate::state::loop_blend::LoopBlend;

/// What presentation reads after a tick: the loop's time and the game state's own frame. The one way state leaves the
/// simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopFrame<F> {
  pub tick: u64,
  /// Game seconds since midnight.
  pub time_of_day: f32,
  /// Game seconds a simulated second.
  pub time_factor: f32,
  pub state: F,
}

impl<F: LoopBlend> LoopBlend for LoopFrame<F> {
  /// The day runs forward across midnight; the tick and the factor are the next frame's once past this one.
  fn blend(&self, next: &Self, alpha: f32) -> Self {
    let ahead: f32 = (next.time_of_day - self.time_of_day).rem_euclid(GameClock::DAY);
    let base: &Self = if alpha > 0.0 { next } else { self };

    Self {
      tick: base.tick,
      time_of_day: (self.time_of_day + ahead * alpha).rem_euclid(GameClock::DAY),
      time_factor: base.time_factor,
      state: self.state.blend(&next.state, alpha),
    }
  }
}
