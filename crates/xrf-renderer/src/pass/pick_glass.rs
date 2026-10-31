/// What a pick does with glass and the other composited surfaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PickGlass {
  /// Glass is the nearest surface a click meets.
  Meets,
  /// The pick passes glass by, to what stands behind it.
  PassesBy,
}

impl PickGlass {
  /// A pick's way with glass where it passes it by (`is_through`) or not.
  pub fn of(is_through: bool) -> Self {
    if is_through { Self::PassesBy } else { Self::Meets }
  }
}
