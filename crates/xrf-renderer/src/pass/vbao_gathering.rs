/// What the visibility bitmask's search and accumulation gather: the occlusion alone, or the light beside it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VbaoGathering {
  Occlusion,
  Light,
}

impl VbaoGathering {
  /// The light where `is_lit`, else the occlusion alone.
  pub fn of(is_lit: bool) -> Self {
    if is_lit { Self::Light } else { Self::Occlusion }
  }
}
