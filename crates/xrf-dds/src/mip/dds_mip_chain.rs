use image::RgbaImage;
use xrf_error::{XrfError, XrfResult};

use crate::mip::dds_mip_resampler::DdsMipResampler;
use crate::mip::dds_mipmaps::DdsMipmaps;

/// A base image and the reduced levels below it, ready to be encoded as they are.
///
/// Every level is reduced from the base rather than from the level above it, so a wide kernel is applied once instead
/// of compounding its own output. For a box kernel the two would be the same picture, since averaging four 2x2 averages
/// is averaging 4x4, but for each level being rounded to bytes before the next is reduced from it.
///
/// Channels are resampled independently and alpha is never premultiplied. That is wrong for a picture whose
/// transparent texels carry no colour, and right for this format: X-Ray packs a bump pair's normal, gloss, error and
/// height into the four channels of two textures, and premultiplying would fold one plane into another.
pub struct DdsMipChain {
  levels: Vec<RgbaImage>,
}

impl DdsMipChain {
  /// The levels a texture is written with, from the base down to the level `mipmaps` stops at.
  ///
  /// # Errors
  ///
  /// Returns an error when the base has no pixels, which nothing downstream can encode.
  pub fn build(base: &RgbaImage, mipmaps: DdsMipmaps) -> XrfResult<Self> {
    if base.width() == 0 || base.height() == 0 {
      return Err(XrfError::new_texture_processing_error(
        "Cannot build a mip chain from an image with no pixels",
      ));
    }

    let mut levels: Vec<RgbaImage> = vec![base.clone()];

    if let DdsMipmaps::Filtered(filter) = mipmaps {
      for level in 1..Self::level_count(base.width(), base.height()) {
        let size: (u32, u32) = Self::level_size(base.width(), base.height(), level);

        levels.push(DdsMipResampler::new(base.dimensions(), size, filter).resample(base));
      }
    }

    Ok(Self { levels })
  }

  /// A chain a caller reduced itself, base first, for a converter whose levels are its own rather than a kernel's.
  ///
  /// The chain may stop early, as a DDS may, but each level must be the size [`Self::build`] would give it, since a
  /// level of any other size is not one the format can hold at that place.
  ///
  /// # Errors
  ///
  /// Returns an error when there is no base, the base has no pixels, or a level is not half the size of the one above.
  pub fn from_levels(levels: Vec<RgbaImage>) -> XrfResult<Self> {
    let Some(base) = levels.first() else {
      return Err(XrfError::new_texture_processing_error(
        "Cannot build a mip chain holding no levels",
      ));
    };
    let (width, height): (u32, u32) = base.dimensions();

    if width == 0 || height == 0 {
      return Err(XrfError::new_texture_processing_error(
        "Cannot build a mip chain from an image with no pixels",
      ));
    }

    if levels.len() > Self::level_count(width, height) as usize {
      return Err(XrfError::new_texture_processing_error(format!(
        "A {width}x{height} base carries at most {} levels, not {}",
        Self::level_count(width, height),
        levels.len()
      )));
    }

    for (level, image) in (0u32..).zip(&levels) {
      let expected: (u32, u32) = Self::level_size(width, height, level);

      if image.dimensions() != expected {
        return Err(XrfError::new_texture_processing_error(format!(
          "Mip level {level} of a {width}x{height} base is {}x{}, not {}x{}",
          image.width(),
          image.height(),
          expected.0,
          expected.1
        )));
      }
    }

    Ok(Self { levels })
  }

  /// The levels, base first.
  pub fn levels(&self) -> &[RgbaImage] {
    &self.levels
  }

  /// The size of one mip level of a base this size, which halves on each axis and never falls below one pixel.
  ///
  /// The one rule for it, shared by building a chain and by reading one back out of a payload.
  pub(crate) fn level_size(width: u32, height: u32, level: u32) -> (u32, u32) {
    ((width >> level).max(1), (height >> level).max(1))
  }

  /// Levels a texture of this size carries, counting the base.
  fn level_count(width: u32, height: u32) -> u32 {
    width.max(height).ilog2() + 1
  }
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use xrf_error::XrfResult;

  use super::DdsMipChain;
  use crate::mip::dds_mip_filter::DdsMipFilter;
  use crate::mip::dds_mipmaps::DdsMipmaps;

  /// A row of grey steps, opaque, so every figure below is one channel of arithmetic.
  fn steps(values: &[u8]) -> RgbaImage {
    RgbaImage::from_fn(values.len() as u32, 1, |x, _| {
      let value: u8 = values[x as usize];

      Rgba([value, value, value, u8::MAX])
    })
  }

  /// The red channel of one level, which for [`steps`] is the whole picture.
  fn reds(chain: &DdsMipChain, level: usize) -> Vec<u8> {
    chain.levels()[level].pixels().map(|pixel| pixel.0[0]).collect()
  }

  #[test]
  fn builds_a_level_for_every_halving() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(&steps(&[0, 10, 20, 30]), DdsMipmaps::Filtered(DdsMipFilter::Box))?;

    assert_eq!(
      chain
        .levels()
        .iter()
        .map(|level| level.dimensions())
        .collect::<Vec<(u32, u32)>>(),
      vec![(4, 1), (2, 1), (1, 1)],
      "Expect the shorter axis to stop at one pixel while the longer one keeps halving"
    );

    // A square reduces on both axes at once, and an oblong keeps going until its longer side runs out.
    assert_eq!(
      DdsMipChain::build(&RgbaImage::new(4, 4), DdsMipmaps::Filtered(DdsMipFilter::Box))?
        .levels()
        .len(),
      3
    );
    assert_eq!(
      DdsMipChain::build(&RgbaImage::new(8, 2), DdsMipmaps::Filtered(DdsMipFilter::Box))?
        .levels()
        .len(),
      4
    );
    assert_eq!(
      DdsMipChain::build(&RgbaImage::new(1, 1), DdsMipmaps::Filtered(DdsMipFilter::Box))?
        .levels()
        .len(),
      1
    );

    Ok(())
  }

  #[test]
  fn every_level_is_reduced_from_the_base() -> XrfResult {
    // The distinguishing case: reducing the base straight to one pixel puts the centre at source pixel 2, while
    // reducing twice in a row would land on 3. A wide kernel applied once, rather than to its own output.
    let chain: DdsMipChain = DdsMipChain::build(&steps(&[0, 10, 20, 30]), DdsMipmaps::Filtered(DdsMipFilter::Point))?;

    assert_eq!(reds(&chain, 2), vec![20]);

    Ok(())
  }

  #[test]
  fn a_flat_chain_carries_the_base_alone() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(&steps(&[0, 10, 20, 30]), DdsMipmaps::Disabled)?;

    assert_eq!(chain.levels().len(), 1);

    Ok(())
  }

  #[test]
  fn takes_a_callers_own_levels_when_each_halves_the_one_above() -> XrfResult {
    // A chain stopping early, as the SDK's own reduction does once the shorter side reaches one.
    let chain: DdsMipChain = DdsMipChain::from_levels(vec![RgbaImage::new(8, 2), RgbaImage::new(4, 1)])?;

    assert_eq!(chain.levels().len(), 2);
    assert!(DdsMipChain::from_levels(vec![RgbaImage::new(8, 2), RgbaImage::new(3, 1)]).is_err());
    assert!(DdsMipChain::from_levels(vec![RgbaImage::new(1, 1), RgbaImage::new(1, 1)]).is_err());
    assert!(DdsMipChain::from_levels(Vec::new()).is_err());

    Ok(())
  }

  #[test]
  fn refuses_an_image_with_no_pixels() {
    assert!(DdsMipChain::build(&RgbaImage::new(0, 4), DdsMipmaps::Filtered(DdsMipFilter::Box)).is_err());
    assert!(DdsMipChain::build(&RgbaImage::new(4, 0), DdsMipmaps::Filtered(DdsMipFilter::Box)).is_err());
  }
}
