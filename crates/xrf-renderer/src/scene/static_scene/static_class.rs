/// How a static draw is shaded, which is what splits draws into batches beside their layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StaticClass {
  /// Every texel written into the G-buffer.
  Opaque,
  /// Texels at or below the surface's alpha reference killed.
  CutOut,
  /// Drawn over the lit frame by the water's own pass, casting no shadow.
  Water,
}

impl StaticClass {
  pub const ALL: [StaticClass; 3] = [StaticClass::Opaque, StaticClass::CutOut, StaticClass::Water];

  pub const fn get_index(self) -> usize {
    match self {
      StaticClass::Opaque => 0,
      StaticClass::CutOut => 1,
      StaticClass::Water => 2,
    }
  }

  /// Whether it is drawn into the G-buffer and casts shadows.
  pub const fn is_deferred(self) -> bool {
    !matches!(self, StaticClass::Water)
  }
}
