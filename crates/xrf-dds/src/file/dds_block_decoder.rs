//! Decodes one level of a flat block-compressed surface in stripes, on every worker at once.

use ddsfile::Dds;
use image::RgbaImage;
use image_dds::{ImageFormat, Surface, mip_dimension};
use xrf_job::parallel::prelude::*;

use crate::encode::dds_image_squares::DdsImageSquares;
use crate::file::dds_block_layout::DdsBlockLayout;

/// The decoder [`crate::DdsFile::decode_rgba`] tries first, for block-compressed layouts.
///
/// A block decodes from its own bytes alone, so a level decoded a stripe at a time is the pixels the whole level decodes
/// to, and a stripe can be used and let go before the next is decoded.
pub(crate) struct DdsBlockDecoder<'a> {
  format: ImageFormat,
  /// The level's blocks, row of blocks after row of blocks.
  blocks: &'a [u8],
  width: u32,
  height: u32,
  /// Bytes of one row of blocks.
  block_row_bytes: usize,
}

impl<'a> DdsBlockDecoder<'a> {
  /// Rows of a level decoded as one piece, a whole number of blocks high.
  const STRIPE_ROWS: u32 = 16 * DdsBlockLayout::SIDE;

  /// One level of a flat BC1 to BC7 texture, or `None` for a cube, an array, a volume, another layout or a payload
  /// `image_dds` refuses, which the caller then decodes whole.
  pub(crate) fn decode(dds: &Dds, mipmap_level: u32) -> Option<RgbaImage> {
    let decoder: DdsBlockDecoder<'_> = DdsBlockDecoder::of(dds, mipmap_level)?;
    let stripe_bytes: usize = Self::STRIPE_ROWS as usize * decoder.width as usize * 4;
    let mut image: RgbaImage = RgbaImage::new(decoder.width, decoder.height);
    let pixels: &mut [u8] = &mut image;
    let is_decoded: bool = pixels
      .par_chunks_mut(stripe_bytes)
      .zip(decoder.list_stripes())
      .all(|(stripe, first)| match decoder.decode_stripe(first) {
        Some(decoded) if decoded.len() == stripe.len() => {
          stripe.copy_from_slice(&decoded);
          true
        }
        _ => false,
      });

    is_decoded.then_some(image)
  }

  /// One level of a flat BC1 to BC7 texture compared with `reference` stripe by stripe, never decoded whole; `None`
  /// where [`Self::decode`] would answer none, or `reference` is not the level's size, which the caller then compares
  /// whole.
  pub(crate) fn compare(dds: &Dds, mipmap_level: u32, reference: &RgbaImage) -> Option<DdsImageSquares> {
    let decoder: DdsBlockDecoder<'_> = DdsBlockDecoder::of(dds, mipmap_level)?;

    if reference.dimensions() != (decoder.width, decoder.height) {
      return None;
    }

    let row_bytes: usize = decoder.width as usize * 4;

    decoder
      .list_stripes()
      .map(|first| {
        let decoded: Vec<u8> = decoder.decode_stripe(first)?;
        let start: usize = first as usize * row_bytes;
        let reference: &[u8] = reference.as_raw().get(start..start + decoded.len())?;

        (decoded.len() == reference.len()).then(|| DdsImageSquares::of(reference, &decoded))
      })
      .try_reduce(DdsImageSquares::default, |left, right| Some(left.add(right)))
  }

  /// The decoder of one level of a flat BC1 to BC7 texture; `None` where [`Self::decode`] answers none.
  fn of(dds: &'a Dds, mipmap_level: u32) -> Option<Self> {
    let surface: Surface<&'a [u8]> = Surface::from_dds(dds).ok()?;

    // A header claiming no texels leaves no stripe to split; the whole-level path answers its error.
    if surface.layers != 1
      || surface.depth != 1
      || surface.width == 0
      || surface.height == 0
      || mipmap_level >= surface.mipmaps
    {
      return None;
    }

    let block_bytes: usize = DdsBlockLayout::block_bytes(surface.image_format)?;
    let width: u32 = mip_dimension(surface.width, mipmap_level);
    // `Surface::get` lends the level for the surface value's life alone; the same bytes are taken again out of the
    // payload, which lives as long as `dds`.
    let level: &[u8] = surface.get(0, 0, mipmap_level)?;
    let start: usize = (level.as_ptr() as usize).checked_sub(surface.data.as_ptr() as usize)?;
    let blocks: &'a [u8] = surface.data.get(start..start + level.len())?;

    Some(Self {
      format: surface.image_format,
      blocks,
      width,
      height: mip_dimension(surface.height, mipmap_level),
      block_row_bytes: DdsBlockLayout::block_row_bytes(width, block_bytes),
    })
  }

  /// The first row of each stripe, on every worker.
  fn list_stripes(&self) -> impl IndexedParallelIterator<Item = u32> {
    (0..self.height).into_par_iter().step_by(Self::STRIPE_ROWS as usize)
  }

  /// The pixels of the stripe starting at row `first`, or `None` for blocks the payload is short of or `image_dds`
  /// refuses.
  fn decode_stripe(&self, first: u32) -> Option<Vec<u8>> {
    let rows: u32 = Self::STRIPE_ROWS.min(self.height - first);
    let start: usize = (first / DdsBlockLayout::SIDE) as usize * self.block_row_bytes;
    let data: &[u8] = self
      .blocks
      .get(start..start + rows.div_ceil(DdsBlockLayout::SIDE) as usize * self.block_row_bytes)?;

    Surface {
      width: self.width,
      height: rows,
      depth: 1,
      layers: 1,
      mipmaps: 1,
      image_format: self.format,
      data,
    }
    .decode_rgba8()
    .ok()
    .map(|surface| surface.data)
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
  use crate::encode::dds_image_squares::DdsImageSquares;
  use crate::mip::dds_mip_chain::DdsMipChain;
  use crate::mip::dds_mip_filter::DdsMipFilter;
  use crate::mip::dds_mipmaps::DdsMipmaps;

  /// A picture whose every channel varies, at sides that are not whole blocks and taller than one stripe.
  fn varied() -> RgbaImage {
    let byte = |value: u32| u8::try_from(value % 256).unwrap_or_default();

    RgbaImage::from_fn(203, 141, |x, y| {
      Rgba([byte(x * 7 + y * 3), byte(x * y), byte(x ^ y), byte(x + 2 * y)])
    })
  }

  #[test]
  fn decodes_every_level_as_image_dds_decodes_it_whole() -> XrfResult {
    // Sides that are not whole blocks and levels taller than one stripe, in every block layout the encoder writes.
    let chain: DdsMipChain = DdsMipChain::build(varied(), DdsMipmaps::Filtered(DdsMipFilter::Box))?;

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
  fn compares_in_stripes_as_the_whole_decoded_level_compares() -> XrfResult {
    let reference: RgbaImage = varied();
    let chain: DdsMipChain = DdsMipChain::build(reference.clone(), DdsMipmaps::Disabled)?;

    for format in [
      ImageFormat::BC1RgbaUnorm,
      ImageFormat::BC3RgbaUnorm,
      ImageFormat::BC7RgbaUnorm,
    ] {
      let bytes: Vec<u8> = DdsEncoding::new(format, Quality::Fast)
        .encode(&chain)?
        .write_to_bytes()?;
      let dds: Dds = Dds::read(&mut bytes.as_slice()).expect("expect the DDS to read back");
      let whole: RgbaImage = image_dds::image_from_dds(&dds, 0).expect("expect image_dds to decode the level");

      assert_eq!(
        DdsBlockDecoder::compare(&dds, 0, &reference),
        Some(DdsImageSquares::of(reference.as_raw(), whole.as_raw())),
        "{format:?}"
      );
    }

    let bytes: Vec<u8> = DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast)
      .encode(&chain)?
      .write_to_bytes()?;
    let dds: Dds = Dds::read(&mut bytes.as_slice()).expect("expect the DDS to read back");

    assert_eq!(
      DdsBlockDecoder::compare(&dds, 0, &RgbaImage::new(8, 8)),
      None,
      "a reference of another size is left to the caller"
    );

    Ok(())
  }

  #[test]
  fn leaves_an_uncompressed_layout_to_the_caller() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(RgbaImage::new(8, 8), DdsMipmaps::Disabled)?;
    let bytes: Vec<u8> = DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
      .encode(&chain)?
      .write_to_bytes()?;

    let dds: Dds = Dds::read(&mut bytes.as_slice()).expect("expect the DDS to read back");

    assert_eq!(DdsBlockDecoder::decode(&dds, 0), None);

    Ok(())
  }

  #[test]
  fn leaves_a_header_claiming_no_texels_to_the_caller() -> XrfResult {
    let chain: DdsMipChain = DdsMipChain::build(RgbaImage::new(8, 8), DdsMipmaps::Disabled)?;
    let bytes: Vec<u8> = DdsEncoding::new(ImageFormat::BC1RgbaUnorm, Quality::Fast)
      .encode(&chain)?
      .write_to_bytes()?;
    let mut dds: Dds = Dds::read(&mut bytes.as_slice()).expect("expect the DDS to read back");

    dds.header.width = 0;

    assert_eq!(DdsBlockDecoder::decode(&dds, 0), None);

    Ok(())
  }
}
