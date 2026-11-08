use crate::scene::level::level_time::LevelTime;

/// The level's clock as one view's frame reads it: the time the frame is drawn at, which every animation takes its
/// phase of, and the time the view's last frame was drawn at, which everything stepped on from frame to frame takes its
/// step from. None before the view's first frame and once its motion starts over (`SceneView::restart_motion`), so
/// every such unit starts again at the same frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ViewTime {
  pub now: LevelTime,
  pub previous: Option<LevelTime>,
}

impl ViewTime {
  /// Seconds from the view's last frame to this one, negative where the clock ran back, as it does for a burst drawn
  /// at an earlier time; none on a first frame.
  pub(crate) fn get_step(&self) -> Option<f64> {
    self
      .previous
      .map(|previous| self.now.get_seconds() - previous.get_seconds())
  }

  /// Seconds a sum stepped every frame moves on by: the step forward, none where the clock ran back or on a first
  /// frame, so a summed clock never runs back and starts again where the view does.
  pub(crate) fn get_advance(&self) -> f64 {
    self.get_step().map_or(0.0, |step| step.max(0.0))
  }

  /// The time the last frame was drawn at, this frame's on a first frame, where something stood still since.
  pub(crate) fn get_previous(&self) -> LevelTime {
    self.previous.unwrap_or(self.now)
  }
}
