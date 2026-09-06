use ddsfile::Dds;
use image_dds::{ImageFormat, Mipmaps, Quality};
use xrf_error::{XrfError, XrfResult};

use crate::file::dds_file::DdsFile;
use crate::mip::dds_mip_chain::DdsMipChain;

/// How a texture is written: the layout its pixels are packed into, and how hard the encoder works at it.
///
/// The one door out of this crate for producing a DDS. It takes levels rather than an image because the levels are
/// the expensive part and are worth keeping: weighing five formats against one texture is one [`DdsMipChain`] and five
/// encodings of it, and a door taking an image would have reduced the same picture five times.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DdsEncoding {
  pub format: ImageFormat,
  pub quality: Quality,
}

impl DdsEncoding {
  pub fn new(format: ImageFormat, quality: Quality) -> Self {
    Self { format, quality }
  }

  /// Writes the chain into a texture, level for level.
  ///
  /// The encoder is told to take the levels it is handed rather than produce its own: the X-Ray converter's filter
  /// family lives in [`crate::DdsMipFilter`] and has no counterpart in `image_dds`, whose generator is a fixed box
  /// reduction.
  ///
  /// # Errors
  ///
  /// Returns an error when the chain cannot be laid out as a surface, or when the encoder refuses the pixels.
  pub fn encode(&self, chain: &DdsMipChain) -> XrfResult<DdsFile> {
    let dds: Dds = chain
      .to_surface()?
      .encode(self.format, self.quality, Mipmaps::FromSurface)
      .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?
      .to_dds()
      .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

    DdsFile::from_encoded(dds)
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use image_dds::{ImageFormat, Quality};
  use xrf_error::XrfResult;

  use super::DdsEncoding;
  use crate::file::dds_file::DdsFile;
  use crate::mip::dds_mip_chain::DdsMipChain;
  use crate::mip::dds_mip_filter::DdsMipFilter;
  use crate::mip::dds_mipmaps::DdsMipmaps;

  fn encode(width: u32, height: u32, mipmaps: DdsMipmaps) -> XrfResult<DdsFile> {
    let base: RgbaImage = RgbaImage::from_fn(width, height, |x, y| Rgba([(x * 16) as u8, (y * 32) as u8, 0, u8::MAX]));

    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast).encode(&DdsMipChain::build(&base, mipmaps)?)
  }

  #[test]
  fn writes_exactly_the_levels_the_chain_holds() -> XrfResult {
    // The encoder is handed the levels rather than asked to make its own, so the file's mip count is the chain's
    // length and nothing else. A texture 1023 wide reduces nine times before it runs out.
    assert_eq!(
      encode(1023, 1020, DdsMipmaps::Filtered(DdsMipFilter::Kaiser))?
        .metadata()
        .mipmap_levels,
      10
    );
    assert_eq!(encode(1023, 1020, DdsMipmaps::Disabled)?.metadata().mipmap_levels, 1);

    Ok(())
  }

  #[test]
  fn keeps_dimensions_that_are_not_multiples_of_four() -> XrfResult {
    // The block compressor pads each level out to whole 4x4 blocks and records the unpadded size, so a texture keeps
    // the shape it was authored at.
    let metadata = encode(1023, 1020, DdsMipmaps::Disabled)?.metadata();

    assert_eq!((metadata.width, metadata.height), (1023, 1020));

    Ok(())
  }
}
