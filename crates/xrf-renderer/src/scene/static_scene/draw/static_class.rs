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
  /// Composited as `Composited` is, but strictly sorted (`mapSorted`): drawn back to front from the view's own list
  /// after the particles, never as the cull lists it.
  Sorted,
}

impl StaticClass {
  pub const ALL: [Self; 6] = [
    Self::Opaque,
    Self::CutOut,
    Self::Water,
    Self::Composited,
    Self::Wallmark,
    Self::Sorted,
  ];

  pub const fn get_index(self) -> usize {
    match self {
      Self::Opaque => 0,
      Self::CutOut => 1,
      Self::Water => 2,
      Self::Composited => 3,
      Self::Wallmark => 4,
      Self::Sorted => 5,
    }
  }

  /// Whether it is composited over the lit frame, in the cull's order or the view's sorted one.
  pub const fn is_composited(self) -> bool {
    matches!(self, Self::Composited | Self::Sorted)
  }

  /// Whether it is drawn into the G-buffer and casts shadows.
  pub const fn is_deferred(self) -> bool {
    matches!(self, Self::Opaque | Self::CutOut)
  }
}
