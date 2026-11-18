use std::time::Duration;

/// The two clocks a level's effects move by at a frame: the level's, which a burst may set and a function of time
/// reads, and the engine's (`EngineClock`), which never jumps and what steps or waits reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EffectClocks {
  pub level: Duration,
  pub engine: Duration,
}
