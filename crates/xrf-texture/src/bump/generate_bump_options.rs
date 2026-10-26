use std::path::PathBuf;

use image::RgbaImage;
use xrf_dds::{DdsMipFilter, Quality};
use xrf_job::JobHandle;
use xrf_thm::ThmBumpChunk;

use crate::bump::generate_bump_gloss::GenerateBumpGloss;

/// What a bump pair is generated from.
///
/// The height source is required and everything else refines it, so a caller with only a height map still gets a
/// usable pair. The SDK reads one source, height in its colour and gloss in its alpha
/// ([`GenerateBumpGloss::HeightAlpha`]); a constant or a mask of its own stands in for a source without one.
pub struct GenerateBumpOptions {
  /// What the run reports its steps to and reads its cancellation from.
  ///
  /// [`JobHandle::inert`] for a caller with nobody watching, which is what the CLI passes.
  pub job: JobHandle,
  /// The bump the pair is written as, without an extension: `<bump>.dds` and `<bump>#.dds` beside it. For a texture's
  /// own bump, [`crate::GenerateBumpProcessor::bump_of`] names it.
  pub bump: PathBuf,
  /// The surface's relief. The normals read the average of its three colour channels, as `AverageRGBToAlpha` stores
  /// it, and the height the second half carries reads its red channel, as `DXTCompressBump` does.
  pub height: RgbaImage,
  pub gloss: GenerateBumpGloss,
  /// Normals to use instead of the ones the height implies, for a descriptor naming an external normal map.
  ///
  /// Must match the height in size. The SDK refuses anything else, and so does this.
  pub normal_map: Option<RgbaImage>,
  /// `bump_virtual_height` of the descriptor, read here and nowhere at runtime.
  pub virtual_height: f32,
  /// The kernel both halves are reduced with. The SDK always reduces them with a box, `STextureParams`'s default.
  pub mip_filter: DdsMipFilter,
  pub quality: Quality,
}

impl GenerateBumpOptions {
  /// The virtual height the SDK starts a descriptor at, and the divisor the height rescale is measured against.
  pub const DEFAULT_VIRTUAL_HEIGHT: f32 = ThmBumpChunk::DEFAULT_VIRTUAL_HEIGHT;

  /// A constant gloss above the warning threshold, for a surface with no mask to read one from.
  pub const DEFAULT_GLOSS: f32 = 0.5;
}
