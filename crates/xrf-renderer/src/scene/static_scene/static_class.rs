/// How a static draw is shaded into the G-buffer, which is what splits draws into batches beside their layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StaticClass {
  /// Every texel written.
  Opaque,
  /// Texels at or below the surface's alpha reference killed.
  CutOut,
}

impl StaticClass {
  pub const ALL: [StaticClass; 2] = [StaticClass::Opaque, StaticClass::CutOut];

  pub const fn get_index(self) -> usize {
    match self {
      StaticClass::Opaque => 0,
      StaticClass::CutOut => 1,
    }
  }
}
