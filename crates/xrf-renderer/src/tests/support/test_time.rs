use std::time::Duration;

use crate::scene::level::level_time::LevelTime;
use crate::scene::level::view::view_time::ViewTime;

/// Level time `seconds` after opening.
pub(crate) fn at(seconds: f64) -> LevelTime {
  LevelTime::new(Duration::from_secs_f64(seconds))
}

/// A view frame at `seconds` whose last frame was at `previous`; none for a first frame.
pub(crate) fn frame_at(seconds: f64, previous: Option<f64>) -> ViewTime {
  ViewTime {
    now: at(seconds),
    previous: previous.map(at),
  }
}
