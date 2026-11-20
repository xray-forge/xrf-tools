use std::cmp::Ordering;

use crate::scene::level::level_time::LevelTime;

/// Draw priority of a shadow face: a stale face by its light's distance from the eye, nearest first; a current
/// (moved or swaying) face by when it was last drawn, oldest first.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LightFaceRank {
  Distance(f32),
  DrawnAt(LevelTime),
}

impl LightFaceRank {
  /// Orders two ranks; a stale rank sorts before a current one.
  pub(crate) fn compare(self, other: Self) -> Ordering {
    match (self, other) {
      (Self::Distance(a), Self::Distance(b)) => a.total_cmp(&b),
      (Self::DrawnAt(a), Self::DrawnAt(b)) => a.cmp(&b),
      (Self::Distance(_), Self::DrawnAt(_)) => Ordering::Less,
      (Self::DrawnAt(_), Self::Distance(_)) => Ordering::Greater,
    }
  }
}
