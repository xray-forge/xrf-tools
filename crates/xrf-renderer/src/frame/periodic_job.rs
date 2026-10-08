/// Work drawn once in so many frames, offered to the frame's `PeriodicScheduler`: the passes that draw it, how much of it
/// there is, and how long it may still wait.
#[derive(Clone, Copy, Debug)]
pub struct PeriodicJob {
  /// The passes that draw it, as the timer names them; the first names what its cost is learned under.
  pub passes: &'static [&'static str],
  /// Milliseconds a unit costs before the timer measured any.
  pub seed: f32,
  /// How much it draws, in the units its cost is counted by: texels, or one where it is drawn whole.
  pub units: f32,
  /// Frames it has waited for a draw.
  pub waited: u32,
  /// Frames it may wait at most; none for a job drawn this frame whatever the budget.
  pub wait_most: u32,
}

impl PeriodicJob {
  /// A job drawn this frame whatever the budget: one never drawn, or one whose waiting would show.
  pub fn new_now(passes: &'static [&'static str], seed: f32, units: f32) -> Self {
    Self {
      passes,
      seed,
      units,
      waited: 0,
      wait_most: 0,
    }
  }

  /// A job that may wait up to `wait_most` frames, having waited `waited`.
  pub fn new_waiting(passes: &'static [&'static str], seed: f32, units: f32, (waited, wait_most): (u32, u32)) -> Self {
    Self {
      passes,
      seed,
      units,
      waited,
      wait_most: wait_most.max(1),
    }
  }

  pub fn is_now(&self) -> bool {
    self.wait_most == 0
  }

  /// Whether it waited as long as it may.
  pub fn is_overdue(&self) -> bool {
    !self.is_now() && self.waited >= self.wait_most
  }

  /// How near its deadline it stands: the share of its wait it will have used by this frame's end.
  pub fn get_urgency(&self) -> f32 {
    (self.waited + 1) as f32 / self.wait_most.max(1) as f32
  }
}
