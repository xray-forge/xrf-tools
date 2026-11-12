//! The alpha the X-Ray SDK's compilers read from a texture, as their image library, FreeImage 3.9, decodes it.
//!
//! The compilers load an image through FreeImage, take its 32-bit pixels and flip them back into the file's row order,
//! so a texel's alpha is the base level's, top row first. FreeImage interpolates a DXT5 block's alpha rounding to
//! nearest, `((6 - i) * a0 + (1 + i) * a1 + 3) / 7`, where `image_dds` rounds `+ 1` and so reads some texels one step
//! lower; it drops the alpha bits of a 16-bit packing and has none for a 24-bit one, whose texels then read opaque; and
//! it reads no DX10 header, alpha-only or luminance layout at all.

use ddsfile::{Dds, FourCC, PixelFormatFlags};
use image::GrayImage;
use xrf_error::{XrfError, XrfResult};

/// The decoder behind [`crate::DdsFile::decode_sdk_alpha`].
pub(crate) struct DdsSdkAlphaDecoder;

impl DdsSdkAlphaDecoder {
  /// Pixels along one edge of a block.
  const BLOCK_SIDE: u32 = 4;

  /// The base level's alpha, one byte a texel, top row first.
  ///
  /// # Errors
  ///
  /// Returns an error for a layout FreeImage 3.9 does not load, a level with no texels, a block-compressed level whose
  /// sides are not whole blocks, or a payload shorter than the level.
  pub(crate) fn decode(dds: &Dds) -> XrfResult<GrayImage> {
    let format = &dds.header.spf;
    let (width, height) = (dds.get_width(), dds.get_height());

    if dds.header10.is_some() {
      return Err(XrfError::new_texture_processing_error(
        "The SDK's image library reads no DX10 header",
      ));
    }

    if width == 0 || height == 0 {
      return Err(XrfError::new_texture_processing_error(format!(
        "A {width} by {height} level holds no texels"
      )));
    }

    match format.fourcc.as_ref().map(|fourcc| fourcc.0) {
      Some(FourCC::DXT1) => Self::decode_blocks(dds, 8, Self::decode_dxt1_block),
      Some(FourCC::DXT3) => Self::decode_blocks(dds, 16, Self::decode_dxt3_block),
      Some(FourCC::DXT5) => Self::decode_blocks(dds, 16, Self::decode_dxt5_block),
      Some(fourcc) => Err(XrfError::new_texture_processing_error(format!(
        "The SDK's image library does not load the layout {:?}",
        fourcc.to_le_bytes().map(char::from)
      ))),
      None if format.flags.contains(PixelFormatFlags::RGB) => match format.rgb_bit_count {
        Some(32) if format.flags.contains(PixelFormatFlags::ALPHA_PIXELS) => {
          // FreeImage copies each pixel's four bytes as they are, and the alpha it reads is the last of them.
          let level: &[u8] = Self::get_level(dds, width as usize * height as usize * 4)?;

          Ok(GrayImage::from_fn(width, height, |x, y| {
            image::Luma([level[(y * width + x) as usize * 4 + 3]])
          }))
        }
        Some(16 | 24 | 32) => Ok(GrayImage::from_pixel(width, height, image::Luma([u8::MAX]))),
        bits => Err(XrfError::new_texture_processing_error(format!(
          "The SDK's image library does not load an uncompressed layout of {bits:?} bits"
        ))),
      },
      None => Err(XrfError::new_texture_processing_error(
        "The SDK's image library loads no alpha-only or luminance layout",
      )),
    }
  }

  /// Every block of the base level decoded by `decode_block`, which writes one block's sixteen alphas in rows.
  fn decode_blocks(dds: &Dds, block_bytes: usize, decode_block: fn(&[u8], &mut [u8; 16])) -> XrfResult<GrayImage> {
    let (width, height) = (dds.get_width(), dds.get_height());

    if width % Self::BLOCK_SIDE != 0 || height % Self::BLOCK_SIDE != 0 {
      return Err(XrfError::new_texture_processing_error(format!(
        "A {width} by {height} level is not whole blocks, whose edges FreeImage 3.9 is not known to decode"
      )));
    }

    let blocks_wide: u32 = width / Self::BLOCK_SIDE;
    let blocks_high: u32 = height / Self::BLOCK_SIDE;
    let level: &[u8] = Self::get_level(dds, blocks_wide as usize * blocks_high as usize * block_bytes)?;
    let mut image: GrayImage = GrayImage::new(width, height);
    let mut alphas: [u8; 16] = [0; 16];

    for (index, block) in level.chunks_exact(block_bytes).enumerate() {
      let left: u32 = index as u32 % blocks_wide * Self::BLOCK_SIDE;
      let top: u32 = index as u32 / blocks_wide * Self::BLOCK_SIDE;

      decode_block(block, &mut alphas);

      for (texel, alpha) in alphas.iter().enumerate() {
        image.put_pixel(
          left + texel as u32 % Self::BLOCK_SIDE,
          top + texel as u32 / Self::BLOCK_SIDE,
          image::Luma([*alpha]),
        );
      }
    }

    Ok(image)
  }

  /// A DXT1 block: its fourth colour is transparent where the first endpoint is not above the second, and every other
  /// texel opaque.
  fn decode_dxt1_block(block: &[u8], alphas: &mut [u8; 16]) {
    let first: u16 = u16::from_le_bytes([block[0], block[1]]);
    let second: u16 = u16::from_le_bytes([block[2], block[3]]);

    for (texel, alpha) in alphas.iter_mut().enumerate() {
      let index: u8 = (block[4 + texel / 4] >> (texel % 4 * 2)) & 0b11;

      *alpha = if first <= second && index == 3 { 0 } else { u8::MAX };
    }
  }

  /// A DXT3 block: four bits a texel, widened as `bits * 255 / 15`.
  fn decode_dxt3_block(block: &[u8], alphas: &mut [u8; 16]) {
    for (texel, alpha) in alphas.iter_mut().enumerate() {
      let bits: u8 = (block[texel / 2] >> (texel % 2 * 4)) & 0xF;

      *alpha = bits * 17;
    }
  }

  /// A DXT5 block: two endpoints and three bits a texel choosing among them and the values between, FreeImage's way.
  fn decode_dxt5_block(block: &[u8], alphas: &mut [u8; 16]) {
    let first: u32 = u32::from(block[0]);
    let second: u32 = u32::from(block[1]);
    let mut palette: [u32; 8] = [first, second, 0, 0, 0, 0, 0, u32::from(u8::MAX)];

    if first > second {
      for step in 0..6 {
        palette[step as usize + 2] = ((6 - step) * first + (1 + step) * second + 3) / 7;
      }
    } else {
      for step in 0..4 {
        palette[step as usize + 2] = ((4 - step) * first + (1 + step) * second + 2) / 5;
      }
    }

    let indices: u64 = block[2..8]
      .iter()
      .rev()
      .fold(0, |bits, byte| (bits << 8) | u64::from(*byte));

    for (texel, alpha) in alphas.iter_mut().enumerate() {
      *alpha = palette[((indices >> (texel * 3)) & 0b111) as usize] as u8;
    }
  }

  /// The base level's bytes, which come first in the payload.
  fn get_level(dds: &Dds, length: usize) -> XrfResult<&[u8]> {
    dds.data.get(..length).ok_or_else(|| {
      XrfError::new_texture_processing_error(format!(
        "DDS payload holds {} bytes, and its base level needs {length}",
        dds.data.len()
      ))
    })
  }
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use ddsfile::{Dds, FourCC};
  use image::{GrayImage, RgbaImage};
  use image_dds::{ImageFormat, Quality};
  use xrf_error::XrfResult;

  use super::DdsSdkAlphaDecoder;
  use crate::encode::dds_encoding::DdsEncoding;
  use crate::mip::dds_mip_chain::DdsMipChain;
  use crate::mip::dds_mipmaps::DdsMipmaps;

  /// A DDS of `width` by `height` with the pixel format's flags, FourCC, bit count and masks, then `payload`.
  fn dds(size: [u32; 2], flags: u32, fourcc: u32, bits: u32, masks: [u32; 4], payload: &[u8]) -> Dds {
    let mut header: [u32; 31] = [0; 31];

    header[0] = 124;
    header[1] = 0x1 | 0x2 | 0x4 | 0x1000;
    header[2] = size[1];
    header[3] = size[0];
    header[18] = 32;
    header[19] = flags;
    header[20] = fourcc;
    header[21] = bits;
    header[22..26].copy_from_slice(&masks);
    header[26] = 0x1000;

    let bytes: Vec<u8> = [0x2053_4444]
      .iter()
      .chain(header.iter())
      .flat_map(|value: &u32| value.to_le_bytes())
      .chain(payload.iter().copied())
      .collect();

    Dds::read(&mut Cursor::new(bytes)).expect("expect the crafted header to parse")
  }

  /// A DXT5 block of two alpha endpoints whose sixteen texels all choose `index`, its colour black.
  fn dxt5_block(endpoints: [u8; 2], index: u64) -> Vec<u8> {
    let indices: u64 = (0..16).fold(0, |bits, texel| bits | (index << (texel * 3)));
    let mut block: Vec<u8> = endpoints.to_vec();

    block.extend_from_slice(&indices.to_le_bytes()[..6]);
    block.extend_from_slice(&[0; 8]);

    block
  }

  fn alphas(image: &GrayImage) -> Vec<u8> {
    image.pixels().map(|pixel| pixel.0[0]).collect()
  }

  #[test]
  fn rounds_dxt5_alpha_to_nearest_where_image_dds_rounds_down() -> XrfResult {
    let dds: Dds = dds([4, 4], 0x4, FourCC::DXT5, 0, [0; 4], &dxt5_block([255, 0], 2));
    assert_eq!(alphas(&DdsSdkAlphaDecoder::decode(&dds)?), vec![219; 16]);
    assert_eq!(image_dds_alphas(&dds), vec![218; 16]);

    Ok(())
  }

  #[test]
  fn reads_dxt5_six_value_blocks_with_their_two_fixed_ends() -> XrfResult {
    for (index, alpha) in [(2, 51), (5, 204), (6, 0), (7, 255)] {
      let dds: Dds = dds([4, 4], 0x4, FourCC::DXT5, 0, [0; 4], &dxt5_block([0, 255], index));

      assert_eq!(
        alphas(&DdsSdkAlphaDecoder::decode(&dds)?),
        vec![alpha; 16],
        "index {index}"
      );
    }

    Ok(())
  }

  /// The alpha `image_dds` decodes the base level to.
  fn image_dds_alphas(dds: &Dds) -> Vec<u8> {
    image_dds::image_from_dds(dds, 0)
      .expect("expect image_dds to decode the level")
      .pixels()
      .map(|pixel| pixel.0[3])
      .collect()
  }

  #[test]
  fn reads_dxt1_alpha_as_image_dds_does() -> XrfResult {
    // The texels cycle through the four indices; the fourth is transparent only where the first endpoint is not above
    // the second, the three-colour mode.
    let indices: [u8; 4] = [0b1110_0100; 4];
    let three_colour: Vec<u8> = [&[0x00, 0x00, 0xff, 0xff][..], &indices].concat();
    let four_colour: Vec<u8> = [&[0xff, 0xff, 0x00, 0x00][..], &indices].concat();

    for (block, transparent) in [(three_colour, 4), (four_colour, 0)] {
      let dds: Dds = dds([4, 4], 0x4, FourCC::DXT1, 0, [0; 4], &block);
      let decoded: Vec<u8> = alphas(&DdsSdkAlphaDecoder::decode(&dds)?);

      assert_eq!(decoded, image_dds_alphas(&dds));
      assert_eq!(decoded.iter().filter(|alpha| **alpha == 0).count(), transparent);
    }

    Ok(())
  }

  #[test]
  fn reads_dxt3_alpha_as_image_dds_does() -> XrfResult {
    let byte = |value: u32| u8::try_from(value % 256).unwrap_or_default();
    let base: RgbaImage = RgbaImage::from_fn(64, 32, |x, y| {
      image::Rgba([byte(x * 9), byte(y * 5), byte(x ^ y), byte(x * 17 + y * 31)])
    });
    let bytes: Vec<u8> = DdsEncoding::new(ImageFormat::BC2RgbaUnorm, Quality::Fast)
      .encode(&DdsMipChain::build(&base, DdsMipmaps::Disabled)?)?
      .write_to_bytes()?;
    let dds: Dds = Dds::read(&mut bytes.as_slice()).expect("expect the DDS to read back");
    let decoded: Vec<u8> = alphas(&DdsSdkAlphaDecoder::decode(&dds)?);

    assert_eq!(decoded, image_dds_alphas(&dds));
    assert!(decoded.iter().any(|alpha| *alpha != u8::MAX));

    Ok(())
  }

  #[test]
  fn places_each_block_at_its_rows_top_first() -> XrfResult {
    let payload: Vec<u8> = [dxt5_block([10, 10], 0), dxt5_block([200, 200], 0)].concat();
    let decoded: GrayImage = DdsSdkAlphaDecoder::decode(&dds([4, 8], 0x4, FourCC::DXT5, 0, [0; 4], &payload))?;

    assert_eq!(decoded.get_pixel(3, 3).0[0], 10);
    assert_eq!(decoded.get_pixel(0, 4).0[0], 200);

    Ok(())
  }

  #[test]
  fn reads_a_thirty_two_bit_pixel_s_last_byte_and_no_alpha_from_narrower_packings() -> XrfResult {
    let pixels: [u8; 16] = [1, 2, 3, 40, 5, 6, 7, 80, 9, 10, 11, 120, 13, 14, 15, 160];
    let argb: [u32; 4] = [0x00ff_0000, 0x0000_ff00, 0x0000_00ff, 0xff00_0000];
    let with_alpha: Dds = dds([2, 2], 0x40 | 0x1, 0, 32, argb, &pixels);
    let without_alpha: Dds = dds([2, 2], 0x40, 0, 32, [argb[0], argb[1], argb[2], 0], &pixels);
    let sixteen_bits: Dds = dds([2, 2], 0x40 | 0x1, 0, 16, [0x7c00, 0x03e0, 0x001f, 0x8000], &[0; 8]);

    assert_eq!(
      alphas(&DdsSdkAlphaDecoder::decode(&with_alpha)?),
      vec![40, 80, 120, 160]
    );
    assert_eq!(alphas(&DdsSdkAlphaDecoder::decode(&without_alpha)?), vec![255; 4]);
    assert_eq!(alphas(&DdsSdkAlphaDecoder::decode(&sixteen_bits)?), vec![255; 4]);

    Ok(())
  }

  #[test]
  fn refuses_what_the_sdk_s_image_library_does_not_read() {
    let alpha_only: Dds = dds([2, 2], 0x2, 0, 8, [0, 0, 0, 0xff], &[0; 4]);
    let partial_blocks: Dds = dds([2, 2], 0x4, FourCC::DXT5, 0, [0; 4], &dxt5_block([0, 0], 0));
    let short: Dds = dds([8, 8], 0x4, FourCC::DXT5, 0, [0; 4], &dxt5_block([0, 0], 0));
    let empty: Dds = dds([0, 4], 0x4, FourCC::DXT5, 0, [0; 4], &[]);

    for dds in [alpha_only, partial_blocks, short, empty] {
      assert!(DdsSdkAlphaDecoder::decode(&dds).is_err());
    }
  }
}
