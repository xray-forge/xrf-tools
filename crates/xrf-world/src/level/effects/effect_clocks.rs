use std::time::Duration;

/// The two clocks a level's effects move by at a frame: the level's (`level`), which a burst sets to times of its own,
/// what a function of time reads; and the engine's (`engine`, `EngineClock`), the level's added up a frame at a time,
/// never back and at most the engine's longest step, what steps or waits by the time passed reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EffectClocks {
  pub level: Duration,
  pub engine: Duration,
}
