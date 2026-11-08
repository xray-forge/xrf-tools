use std::time::Duration;

use crate::scene::level::level_time::LevelTime;
use crate::scene::level::view::view_time::ViewTime;

/// The level's clock `seconds` after it began opening.
pub(crate) fn at(seconds: f64) -> LevelTime {
  LevelTime::new(Duration::from_secs_f64(seconds))
}

/// A view's frame at `seconds` after its last at `previous`, none for its first.
pub(crate) fn frame_at(seconds: f64, previous: Option<f64>) -> ViewTime {
  ViewTime {
    now: at(seconds),
    previous: previous.map(at),
  }
}
