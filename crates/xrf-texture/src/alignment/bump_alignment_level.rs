use xrf_dds::{DdsFile, DdsMetadata, RgbaImage};
use xrf_error::XrfResult;

use crate::alignment::BumpAlignment;

/// One texture of a pair as [`BumpAlignment::compare`] reads it: the largest level within
/// [`BumpAlignment::MEASURED_SIDE`], decoded, with the size of the base level it was reduced from.
pub struct BumpAlignmentLevel {
  /// Texels a side of the texture's base level.
  pub base_size: [u32; 2],
  /// The level compared.
  pub image: RgbaImage,
}

impl BumpAlignmentLevel {
  /// Decodes the level of `file` a comparison reads.
  ///
  /// # Errors
  ///
  /// Returns an error when the level does not decode.
  pub fn read(file: &DdsFile) -> XrfResult<Self> {
    let metadata: DdsMetadata = file.metadata();

    Ok(Self {
      base_size: [metadata.width, metadata.height],
      image: file.decode_rgba(metadata.get_level_within(BumpAlignment::MEASURED_SIDE))?,
    })
  }
}
