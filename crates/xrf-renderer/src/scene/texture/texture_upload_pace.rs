/// How fast loaded textures are uploaded: up to a byte budget a frame while a level opens, or as the frame's scheduler
/// grants them once every level stands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TextureUploadPace {
  Opening,
  Scheduled,
}

impl TextureUploadPace {
  /// The pace while any level `is_opening`, else the scheduled one.
  pub(crate) fn of(is_opening: bool) -> Self {
    if is_opening { Self::Opening } else { Self::Scheduled }
  }
}
