use image::RgbaImage;

/// The gloss the pair carries, which is what the specular response is read out of.
pub enum GenerateBumpGloss {
  /// One value everywhere, for a surface authored without a gloss mask.
  Constant(f32),
  /// A mask, read from the average of its colour channels.
  Mask(RgbaImage),
}
