/// One ray model face's part in a light ray.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ShadowFace {
  Passes,
  Opaque,
  /// By its index among the alpha faces.
  Alpha(u32),
}
