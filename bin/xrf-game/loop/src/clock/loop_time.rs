use std::time::Duration;

/// Where the loop stands as a tick is stepped: the tick and the simulated time a step covers, all a system reads of
/// time from the loop, never the wall clock. Game time, which a save keeps and the game may set, is the game's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopTime {
  /// The tick being stepped from; the step makes it the next.
  pub tick: u64,
  /// The simulated time a step covers.
  pub step: Duration,
}

impl LoopTime {
  /// Time at tick 0, stepping by `step`.
  pub fn new(step: Duration) -> Self {
    Self { tick: 0, step }
  }

  /// The time a step later.
  pub(crate) fn next(&self) -> Self {
    Self {
      tick: self.tick + 1,
      ..*self
    }
  }
}
