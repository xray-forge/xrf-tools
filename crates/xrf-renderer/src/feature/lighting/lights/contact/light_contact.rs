/// Whether the lights' contact shadows were marched this frame, which their visibility and draw read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LightContact {
  Marched,
  Unmarched,
}

impl LightContact {
  /// The contact shadows marched (`is_marched`) or not.
  pub(crate) fn of(is_marched: bool) -> Self {
    if is_marched { Self::Marched } else { Self::Unmarched }
  }

  pub(crate) fn is_marched(self) -> bool {
    self == Self::Marched
  }
}
