/// The state a campfire stands in or turns to: lit, or put out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CampfireState {
  Lit,
  Out,
}

impl CampfireState {
  /// Lit while `is_lit`, else out.
  pub(crate) fn of(is_lit: bool) -> Self {
    if is_lit { Self::Lit } else { Self::Out }
  }
}
