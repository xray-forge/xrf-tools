/// How a static draw is shaded, which is what splits draws into batches beside their layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StaticClass {
  /// Every texel written into the G-buffer.
  Opaque,
  /// Texels at or below the surface's alpha reference killed.
  CutOut,
  /// Drawn over the lit frame by the water's own pass, casting no shadow.
  Water,
  /// Blended, added or multiplied over the lit frame by the composited pass, casting no shadow.
  Composited,
  /// A wall mark, composited into the G-buffer's albedo before any light, casting no shadow.
  Wallmark,
}

impl StaticClass {
  pub const ALL: [StaticClass; 5] = [
    StaticClass::Opaque,
    StaticClass::CutOut,
    StaticClass::Water,
    StaticClass::Composited,
    StaticClass::Wallmark,
  ];

  pub const fn get_index(self) -> usize {
    match self {
      StaticClass::Opaque => 0,
      StaticClass::CutOut => 1,
      StaticClass::Water => 2,
      StaticClass::Composited => 3,
      StaticClass::Wallmark => 4,
    }
  }

  /// Whether it is drawn into the G-buffer and casts shadows.
  pub const fn is_deferred(self) -> bool {
    matches!(self, StaticClass::Opaque | StaticClass::CutOut)
  }
}
