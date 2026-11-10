use std::cmp::Ordering;

use crate::scene::level::level_time::LevelTime;

/// What orders a light's faces wanting a draw within their kind: a stale face by its light's distance from the eye, the
/// nearest first; a current one, moved or swaying, by when it was drawn, the longest ago first.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LightFaceRank {
  Distance(f32),
  DrawnAt(LevelTime),
}

impl LightFaceRank {
  /// The order of two ranks, a stale one before a current one where they are of either kind.
  pub(crate) fn compare(self, other: Self) -> Ordering {
    match (self, other) {
      (Self::Distance(a), Self::Distance(b)) => a.total_cmp(&b),
      (Self::DrawnAt(a), Self::DrawnAt(b)) => a.cmp(&b),
      (Self::Distance(_), Self::DrawnAt(_)) => Ordering::Less,
      (Self::DrawnAt(_), Self::Distance(_)) => Ordering::Greater,
    }
  }
}
