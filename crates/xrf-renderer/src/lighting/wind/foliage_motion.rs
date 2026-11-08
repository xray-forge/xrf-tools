/// Whether foliage moves this frame, the trees' flow fields or the grass's sway: blown on by the wind while the weather
/// sways it, or held still.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FoliageMotion {
  Blowing,
  Still,
}

impl FoliageMotion {
  /// Blowing while `is_windy`, else still.
  pub(crate) fn of(is_windy: bool) -> Self {
    if is_windy { Self::Blowing } else { Self::Still }
  }
}
