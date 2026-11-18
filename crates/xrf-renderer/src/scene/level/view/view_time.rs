use crate::scene::level::level_time::LevelTime;

/// The level's clock for one view's frame: `now` for animation phases, `previous` (the view's last frame) for
/// per-frame steps.
///
/// `previous` is none on the first frame and after `SceneView::restart_motion`, so every stepped unit restarts on the
/// same frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ViewTime {
  pub now: LevelTime,
  pub previous: Option<LevelTime>,
}

impl ViewTime {
  /// Seconds since the view's last frame, negative where the clock ran back (a burst at an earlier time); none on a
  /// first frame.
  pub(crate) fn get_step(&self) -> Option<f64> {
    self
      .previous
      .map(|previous| self.now.get_seconds() - previous.get_seconds())
  }

  /// The forward step for a per-frame sum; zero where the clock ran back or on a first frame, so the sum never
  /// runs back.
  pub(crate) fn get_advance(&self) -> f64 {
    self.get_step().map_or(0.0, |step| step.max(0.0))
  }

  /// The last frame's time, or this frame's on a first frame.
  pub(crate) fn get_previous(&self) -> LevelTime {
    self.previous.unwrap_or(self.now)
  }
}
