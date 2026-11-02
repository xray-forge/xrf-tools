use image::RgbaImage;

/// The gloss the pair carries, which is what the specular response is read out of.
pub enum GenerateBumpGloss {
  /// The height image's own alpha, where the SDK's bump source carries its gloss (`it_gloss_rev`).
  HeightAlpha,
  /// One value everywhere, for a surface authored without a gloss mask.
  Constant(f32),
  /// A mask of its own, read from the average of its colour channels.
  Mask(RgbaImage),
}
