//! Decodes one level of a flat block-compressed surface in stripes, on every worker at once.

use ddsfile::Dds;
use image::RgbaImage;
use image_dds::{ImageFormat, Surface, mip_dimension};
use rayon::prelude::*;

/// The decoder [`crate::DdsFile::decode_rgba`] tries first, for the layouts whose blocks it knows.
pub(crate) struct DdsBlockDecoder;

impl DdsBlockDecoder {
  /// Pixels along one edge of a block of every layout this decodes.
  const BLOCK_SIDE: u32 = 4;
  /// Rows of a level decoded as one piece, a whole number of blocks high.
  const STRIPE_ROWS: u32 = 16 * Self::BLOCK_SIDE;

  /// One level of a flat texture, or `None` where this does not decode it: a cube, an array or a volume, a layout
  /// other than BC1 to BC7, or a payload `image_dds` refuses, each of which the caller decodes as before.
  ///
  /// A block decodes from its own bytes alone, and a level is its rows of blocks in turn, so the stripes laid end to end
  /// are the pixels the whole level decodes to; only the last stripe ends inside a block, as the level does.
  pub(crate) fn decode(dds: &Dds, mipmap_level: u32) -> Option<RgbaImage> {
    let surface: Surface<&[u8]> = Surface::from_dds(dds).ok()?;

    // A header claiming no texels leaves no stripe to split; the whole-level path answers its error.
    if surface.layers != 1
      || surface.depth != 1
      || surface.width == 0
      || surface.height == 0
      || mipmap_level >= surface.mipmaps
    {
      return None;
    }

    let block_bytes: usize = Self::block_bytes(surface.image_format)?;
    let level: &[u8] = surface.get(0, 0, mipmap_level)?;
    let width: u32 = mip_dimension(surface.width, mipmap_level);
    let height: u32 = mip_dimension(surface.height, mipmap_level);
    // Bytes of one row of blocks in the payload, and of one stripe of pixels in the picture.
    let block_row_bytes: usize = width.div_ceil(Self::BLOCK_SIDE) as usize * block_bytes;
    let stripe_bytes: usize = Self::STRIPE_ROWS as usize * width as usize * 4;
    let mut image: RgbaImage = RgbaImage::new(width, height);
    let pixels: &mut [u8] = &mut image;
    let is_decoded: bool = pixels
      .par_chunks_mut(stripe_bytes)
      .zip((0..height).into_par_iter().step_by(Self::STRIPE_ROWS as usize))
      .all(|(stripe, first)| {
        let rows: u32 = Self::STRIPE_ROWS.min(height - first);
        let start: usize = (first / Self::BLOCK_SIDE) as usize * block_row_bytes;
        let blocks: Option<&[u8]> =
          level.get(start..start + rows.div_ceil(Self::BLOCK_SIDE) as usize * block_row_bytes);
        let decoded = blocks.and_then(|data| {
          Surface {
            width,
            height: rows,
            depth: 1,
            layers: 1,
            mipmaps: 1,
            image_format: surface.image_format,
            data,
          }
          .decode_rgba8()
          .ok()
        });

        match decoded {
          Some(decoded) if decoded.data.len() == stripe.len() => {
            stripe.copy_from_slice(&decoded.data);
            true
          }
          _ => false,
        }
      });

    is_decoded.then_some(image)
  }

  /// Bytes a block of a block-compressed layout takes, or `None` for any other layout.
  fn block_bytes(format: ImageFormat) -> Option<usize> {
    use ImageFormat as F;

    match format {
      F::BC1RgbaUnorm | F::BC1RgbaUnormSrgb | F::BC4RUnorm | F::BC4RSnorm => Some(8),
      F::BC2RgbaUnorm
      | F::BC2RgbaUnormSrgb
      | F::BC3RgbaUnorm
      | F::BC3RgbaUnormSrgb
      | F::BC5RgUnorm
      | F::BC5RgSnorm
      | F::BC6hRgbUfloat
      | F::BC6hRgbSfloat
      | F::BC7RgbaUnorm
      | F::BC7RgbaUnormSrgb => Some(16),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use ddsfile::Dds;
  use image::{Rgba, RgbaImage};
  use image_dds::{ImageFormat, Quality};
  use xrf_error::XrfResult;

  use super::DdsBlockDecoder;
  use crate::encode::dds_encoding::DdsEncoding;
  use crate::mip::dds_mip_chain::DdsMipChain;
  use crate::mip::dds_mip_filter::DdsMipFilter;
  use crate::mip::dds_mipmaps::DdsMipmaps;

  #[test]
  fn decodes_every_level_as_image_dds_decodes_it_whole() -> XrfResult {
    // Sides that are not whole blocks and levels taller than one stripe, in every block layout the encoder writes.
    let byte = |value: u32| u8::try_from(value % 256).unwrap_or_default();
    let base: RgbaImage = RgbaImage::from_fn(203, 141, |x, y| {
      Rgba([byte(x * 7 + y * 3), byte(x * y), byte(x ^ y), byte(x + 2 * y)])
    });
    let chain: DdsMipChain = DdsMipChain::build(&base, DdsMipmaps::Filtered(DdsMipFilter::Box))?;

    for format in [
      ImageFormat::BC1RgbaUnorm,
      ImageFormat::BC2RgbaUnorm,
      ImageFormat::BC3RgbaUnorm,
      ImageFormat::BC7RgbaUnorm,
    ] {
      let bytes: Vec<u8> = DdsEncoding::new(format, Quality::Fast)
        .encode(&chain)?
        .write_to_bytes()?;
      let dds: Dds = Dds::read(&mut bytes.as_slice()).expect("expect the DDS to read back");

      for level in 0..dds.get_num_mipmap_levels() {
        let whole: RgbaImage = image_dds::image_from_dds(&dds, level).expect("expect image_dds to decode the level");

        assert_eq!(
          DdsBlockDecoder::decode(&dds, level),
          Some(whole),
          "{format:?} level {level}"
        );
      }
    }

    Ok(())
  }

  #[test]
  fn leaves_an_uncompressed_layout_to_the_caller() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(&RgbaImage::new(8, 8), DdsMipmaps::Disabled)?;
    let bytes: Vec<u8> = DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
      .encode(&chain)?
      .write_to_bytes()?;

    let dds: Dds = Dds::read(&mut bytes.as_slice()).expect("expect the DDS to read back");

    assert_eq!(DdsBlockDecoder::decode(&dds, 0), None);

    Ok(())
  }

  #[test]
  fn leaves_a_header_claiming_no_texels_to_the_caller() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(&RgbaImage::new(8, 8), DdsMipmaps::Disabled)?;
    let bytes: Vec<u8> = DdsEncoding::new(ImageFormat::BC1RgbaUnorm, Quality::Fast)
      .encode(&chain)?
      .write_to_bytes()?;
    let mut dds: Dds = Dds::read(&mut bytes.as_slice()).expect("expect the DDS to read back");

    dds.header.width = 0;

    assert_eq!(DdsBlockDecoder::decode(&dds, 0), None);

    Ok(())
  }
}
