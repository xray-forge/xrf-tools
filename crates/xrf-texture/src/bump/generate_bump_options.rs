use std::path::PathBuf;

use image::RgbaImage;
use xrf_dds::{DdsMipFilter, Quality};
use xrf_job::JobHandle;

/// The gloss the pair carries, which is what the specular response is read out of.
pub enum GenerateBumpGloss {
  /// One value everywhere, for a surface authored without a gloss mask.
  Constant(f32),
  /// A mask, read from the average of its colour channels.
  Mask(RgbaImage),
}

/// What a bump pair is generated from.
///
/// The height source is required and everything else refines it: the SDK derives its normals from the height alone
/// and takes gloss as a separate plane, so a caller with only a height map still gets a usable pair.
pub struct GenerateBumpOptions {
  /// What the run reports its steps to and reads its cancellation from.
  ///
  /// [`JobHandle::inert`] for a caller with nobody watching, which is what the CLI passes.
  pub job: JobHandle,
  /// Where the pair is written, without the `_bump` suffix or an extension.
  pub destination: PathBuf,
  /// The surface's relief. Its three colour channels are averaged, as `AverageRGBToAlpha` does.
  pub height: RgbaImage,
  pub gloss: GenerateBumpGloss,
  /// Normals to use instead of the ones the height implies, for a descriptor naming an external normal map.
  ///
  /// Must match the height in size. The SDK refuses anything else, and so does this.
  pub normal_map: Option<RgbaImage>,
  /// `bump_virtual_height` of the descriptor, read here and nowhere at runtime.
  pub virtual_height: f32,
  pub mip_filter: DdsMipFilter,
  pub quality: Quality,
}

impl GenerateBumpOptions {
  /// The virtual height the SDK starts a descriptor at, and the divisor the height rescale is measured against.
  pub const DEFAULT_VIRTUAL_HEIGHT: f32 = 0.05;

  /// A constant gloss above the warning threshold, for a surface with no mask to read one from.
  pub const DEFAULT_GLOSS: f32 = 0.5;
}
