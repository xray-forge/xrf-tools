use std::time::Duration;

/// A tick on the scene's clock: its time, how far ahead a sound starting is given its voice, and the seconds since the
/// last tick.
#[derive(Clone, Copy)]
pub(crate) struct SceneMoment {
  pub(crate) now: Duration,
  pub(crate) horizon: Duration,
  pub(crate) seconds: f32,
}
