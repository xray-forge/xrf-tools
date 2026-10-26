use image::RgbaImage;

/// The gloss the pair carries, which is what the specular response is read out of.
///
/// The SDK reads it from the alpha of the bump source (`it_gloss_rev`); this takes it as a plane of its own, so that
/// gloss arrives as a mask whose colour is the source's alpha.
pub enum GenerateBumpGloss {
  /// One value everywhere, for a surface authored without a gloss mask.
  Constant(f32),
  /// A mask, read from the average of its colour channels.
  Mask(RgbaImage),
}
