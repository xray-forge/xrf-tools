use image::RgbaImage;
use image_dds::SurfaceRgba8;
use xrf_error::{XrfError, XrfResult};

use crate::mip::dds_mip_resampler::DdsMipResampler;
use crate::mip::dds_mipmaps::DdsMipmaps;

/// A base image and the reduced levels below it, ready to be encoded as they are.
///
/// Every level is reduced from the base rather than from the level above it, so a wide kernel is applied once instead
/// of compounding its own output. For a box kernel the two are the same picture, since averaging four 2x2 averages is
/// averaging 4x4.
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
  /// Built once and encoded as many times as there are formats to weigh, which is what keeps a five-way comparison
  /// from reducing the same picture five times over.
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
        let size: (u32, u32) = (mip_dimension(base.width(), level), mip_dimension(base.height(), level));

        levels.push(DdsMipResampler::new(base.dimensions(), size, filter).resample(base));
      }
    }

    Ok(Self { levels })
  }

  /// The levels, base first.
  pub fn levels(&self) -> &[RgbaImage] {
    &self.levels
  }

  /// The chain as one surface, which is the shape the encoder takes a caller's own levels in.
  ///
  /// # Errors
  ///
  /// Returns an error when the chain describes more levels than the format's `u32` count can carry.
  pub(crate) fn to_surface(&self) -> XrfResult<SurfaceRgba8<Vec<u8>>> {
    let base: &RgbaImage = self
      .levels
      .first()
      .ok_or_else(|| XrfError::new_texture_processing_error("Cannot encode a mip chain holding no levels"))?;

    Ok(SurfaceRgba8 {
      width: base.width(),
      height: base.height(),
      depth: 1,
      layers: 1,
      mipmaps: u32::try_from(self.levels.len())
        .map_err(|_| XrfError::new_texture_processing_error("Mip chain exceeds the supported level count"))?,
      data: self
        .levels
        .iter()
        .flat_map(|level| level.as_raw().iter().copied())
        .collect(),
    })
  }

  /// Levels a texture of this size carries, counting the base.
  fn level_count(width: u32, height: u32) -> u32 {
    width.max(height).ilog2() + 1
  }
}

/// The size of one axis at a mip level, which never falls below one pixel.
fn mip_dimension(base: u32, level: u32) -> u32 {
  (base >> level).max(1)
}

#[cfg(test)]
mod tests {
  use image::{Rgba, RgbaImage};
  use image_dds::SurfaceRgba8;
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
  fn lays_the_chain_out_as_one_surface() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(&RgbaImage::new(4, 4), DdsMipmaps::Filtered(DdsMipFilter::Box))?;
    let surface: SurfaceRgba8<Vec<u8>> = chain.to_surface()?;

    assert_eq!(
      (surface.width, surface.height, surface.depth, surface.layers),
      (4, 4, 1, 1)
    );
    assert_eq!(surface.mipmaps, 3);
    // Every level's pixels, back to back and unpadded: 16 + 4 + 1 pixels of four bytes.
    assert_eq!(surface.data.len(), (16 + 4 + 1) * 4);

    Ok(())
  }

  #[test]
  fn a_flat_chain_carries_the_base_alone() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(&steps(&[0, 10, 20, 30]), DdsMipmaps::Disabled)?;

    assert_eq!(chain.levels().len(), 1);
    assert_eq!(chain.to_surface()?.mipmaps, 1);

    Ok(())
  }

  #[test]
  fn refuses_an_image_with_no_pixels() {
    assert!(DdsMipChain::build(&RgbaImage::new(0, 4), DdsMipmaps::Filtered(DdsMipFilter::Box)).is_err());
    assert!(DdsMipChain::build(&RgbaImage::new(4, 0), DdsMipmaps::Filtered(DdsMipFilter::Box)).is_err());
  }
}
