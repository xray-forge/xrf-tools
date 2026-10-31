/// Whether the lights' contact shadows were marched this frame, which their visibility and draw read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LightContact {
  Marched,
  Unmarched,
}

impl LightContact {
  /// The contact shadows marched (`is_marched`) or not.
  pub fn of(is_marched: bool) -> Self {
    if is_marched { Self::Marched } else { Self::Unmarched }
  }

  pub fn is_marched(self) -> bool {
    self == Self::Marched
  }
}
