//! Expands uncompressed DDS surfaces that `image_dds` has no format for.
//!
//! Every layout here is a packing of channels into 8, 16, 24 or 32 bits, described by the masks in the header rather
//! than by a name, so one mask-driven expansion covers all of them - including the ones nobody has met yet. What X-Ray
//! ships and `image_dds` refuses is `A8` alpha-only (274 files across the reference trees, which is every bitmap font
//! atlas), `R5G6B5` (15), 16 bit alpha-luminance (9), `X8R8G8B8` (4) and `L8` (3).

use ddsfile::{Caps2, Dds, PixelFormatFlags};
use image::RgbaImage;
use xrf_error::{XrfError, XrfResult};

/// One channel's place in a packed pixel: where its bits sit, and how wide they are.
#[derive(Clone, Copy, Debug)]
struct Channel {
  mask: u32,
  shift: u32,
  maximum: u32,
}

impl Channel {
  /// The channel a mask describes, or `None` for a mask that selects no bits.
  fn from_mask(mask: Option<u32>) -> Option<Self> {
    let mask: u32 = mask.filter(|it| *it != 0)?;

    Some(Self {
      mask,
      shift: mask.trailing_zeros(),
      maximum: mask >> mask.trailing_zeros(),
    })
  }

  /// This channel's value in the pixel, widened to eight bits.
  ///
  /// Scaled rather than shifted: five bits of white are `31`, and shifting them left by three gives `248` - a grey
  /// that makes every light surface slightly dirty. Rounding division spreads the range evenly instead.
  fn sample(&self, pixel: u32) -> u8 {
    let value: u32 = (pixel & self.mask) >> self.shift;

    ((value * 255 + self.maximum / 2) / self.maximum) as u8
  }
}

/// How one packed layout maps onto RGBA.
#[derive(Clone, Copy, Debug)]
struct Layout {
  bytes_per_pixel: usize,
  red: Option<Channel>,
  green: Option<Channel>,
  blue: Option<Channel>,
  alpha: Option<Channel>,
  /// Whether the red channel is a luminance the other two repeat.
  is_luminance: bool,
}

impl Layout {
  /// Reads the layout a header declares, or `None` when it declares one this cannot expand.
  ///
  /// `ddsfile` hands back the channel masks only for a header flagged `RGB`, and the bit count only for `RGB` or
  /// `LUMINANCE` - which is every layout this module exists for. What it withholds is recovered rather than guessed,
  /// because the format fixes it: an alpha-only surface has one channel and the alpha mask covers the whole pixel, and
  /// a luminance surface's luminance occupies whatever the alpha mask does not.
  fn from_dds(dds: &Dds) -> Option<Self> {
    let format = &dds.header.spf;

    if format.fourcc.is_some() || dds.header10.is_some() {
      return None;
    }

    let is_luminance: bool = format.flags.contains(PixelFormatFlags::LUMINANCE);
    let is_alpha_only: bool = format.flags.contains(PixelFormatFlags::ALPHA);
    let alpha: Option<Channel> = Channel::from_mask(format.a_bit_mask);

    let bits: u32 = match format.rgb_bit_count {
      Some(bits) => bits,
      // Alpha-only: the pixel is the alpha channel, so its width is the mask's.
      None if is_alpha_only => alpha?.mask.ilog2() + 1,
      None => return None,
    };

    if !matches!(bits, 8 | 16 | 24 | 32) {
      return None;
    }

    let luminance: Option<Channel> = if is_luminance {
      // Everything the alpha mask leaves, within the width of the pixel: `L8` takes the whole byte, and `A8L8` takes
      // the low one because alpha holds the high one.
      let covered: u32 = alpha.map_or(0, |channel| channel.mask);

      Channel::from_mask(Some(pixel_mask(bits) & !covered))
    } else {
      Channel::from_mask(format.r_bit_mask)
    };

    Some(Self {
      bytes_per_pixel: (bits / 8) as usize,
      red: luminance,
      green: Channel::from_mask(format.g_bit_mask),
      blue: Channel::from_mask(format.b_bit_mask),
      alpha,
      is_luminance,
    })
  }

  /// Expands one packed pixel.
  fn to_rgba(self, pixel: u32) -> [u8; 4] {
    let alpha: u8 = self.alpha.map_or(u8::MAX, |channel| channel.sample(pixel));

    let Some(red) = self.red else {
      // No colour channel at all: an alpha-only surface, whose colour is white by the reasoning above.
      return [u8::MAX, u8::MAX, u8::MAX, alpha];
    };

    let red: u8 = red.sample(pixel);

    if self.is_luminance {
      return [red, red, red, alpha];
    }

    [
      red,
      self.green.map_or(0, |channel| channel.sample(pixel)),
      self.blue.map_or(0, |channel| channel.sample(pixel)),
      alpha,
    ]
  }
}

/// Expands one mip level of an uncompressed surface into RGBA.
///
/// # Errors
///
/// Returns an error when the header declares a layout this cannot expand, when the file is a cubemap or volume, or
/// when the payload is shorter than the level asks for.
pub(crate) fn decode_uncompressed(dds: &Dds, mipmap_level: u32) -> XrfResult<RgbaImage> {
  // A cubemap or volume stores faces or slices one after another, and expanding the first as if it were the whole
  // image would show one face stretched over the surface rather than reporting that this is not a flat texture.
  if dds.header.caps2.intersects(Caps2::CUBEMAP | Caps2::VOLUME) {
    return Err(XrfError::new_texture_processing_error(
      "Cannot expand a cubemap or volume texture as a flat image",
    ));
  }

  let layout: Layout = Layout::from_dds(dds).ok_or_else(|| {
    XrfError::new_texture_processing_error("DDS header declares no uncompressed layout that can be expanded")
  })?;

  if mipmap_level >= dds.get_num_mipmap_levels() {
    return Err(XrfError::new_texture_processing_error(format!(
      "DDS carries {} mipmap levels, so level {mipmap_level} cannot be read",
      dds.get_num_mipmap_levels()
    )));
  }

  let (width, height) = level_size(dds.get_width(), dds.get_height(), mipmap_level);
  let mut offset: usize = 0;

  for level in 0..mipmap_level {
    let (level_width, level_height) = level_size(dds.get_width(), dds.get_height(), level);

    offset += level_width as usize * level_height as usize * layout.bytes_per_pixel;
  }

  let length: usize = width as usize * height as usize * layout.bytes_per_pixel;

  if dds.data.len() < offset + length {
    return Err(XrfError::new_texture_processing_error(format!(
      "DDS payload holds {} bytes, and mipmap level {mipmap_level} needs {}",
      dds.data.len(),
      offset + length
    )));
  }

  let mut image: RgbaImage = RgbaImage::new(width, height);

  for (index, pixel) in dds.data[offset..offset + length]
    .chunks_exact(layout.bytes_per_pixel)
    .enumerate()
  {
    let mut packed: u32 = 0;

    // Little endian, as every DDS payload is: the first byte is the least significant part of the pixel.
    for (byte, value) in pixel.iter().enumerate() {
      packed |= u32::from(*value) << (byte * 8);
    }

    image.put_pixel(
      index as u32 % width,
      index as u32 / width,
      image::Rgba(layout.to_rgba(packed)),
    );
  }

  Ok(image)
}

/// Every bit a pixel of this width occupies.
fn pixel_mask(bits: u32) -> u32 {
  if bits >= u32::BITS {
    u32::MAX
  } else {
    (1u32 << bits) - 1
  }
}

/// The size of one mip level, which halves each time and never reaches zero.
fn level_size(width: u32, height: u32, level: u32) -> (u32, u32) {
  ((width >> level).max(1), (height >> level).max(1))
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use ddsfile::Dds;
  use image::RgbaImage;

  use super::decode_uncompressed;

  const DDS_MAGIC: u32 = 0x2053_4444;
  const HEADER_SIZE: u32 = 124;
  const FLAG_CAPS_HEIGHT_WIDTH_PIXELFORMAT: u32 = 0x1 | 0x2 | 0x4 | 0x1000;
  const PIXEL_FORMAT_SIZE: u32 = 32;

  /// Builds a two by two DDS out of a pixel format and its payload, the way the files in the trees are laid out.
  ///
  /// Written byte by byte rather than through a builder so a test states the header it means: these layouts are
  /// distinguished by their masks alone, and a helper that filled masks in would be testing itself.
  fn dds(flags: u32, bits: u32, masks: [u32; 4], payload: &[u8]) -> Dds {
    let mut bytes: Vec<u8> = Vec::new();
    let mut header: [u32; 31] = [0; 31];

    header[0] = HEADER_SIZE;
    header[1] = FLAG_CAPS_HEIGHT_WIDTH_PIXELFORMAT;
    header[2] = 2;
    header[3] = 2;
    header[18] = PIXEL_FORMAT_SIZE;
    header[19] = flags;
    header[21] = bits;
    header[22] = masks[0];
    header[23] = masks[1];
    header[24] = masks[2];
    header[25] = masks[3];
    header[26] = 0x1000;

    bytes.extend_from_slice(&DDS_MAGIC.to_le_bytes());

    for value in header {
      bytes.extend_from_slice(&value.to_le_bytes());
    }

    bytes.extend_from_slice(payload);

    Dds::read(&mut Cursor::new(bytes)).expect("expect the crafted header to parse")
  }

  fn pixels(image: &RgbaImage) -> Vec<[u8; 4]> {
    image.pixels().map(|pixel| pixel.0).collect()
  }

  #[test]
  fn expands_alpha_only_as_coverage_over_white() {
    // `A8` is every bitmap font atlas in the trees. The file states coverage and no colour, and white is the only
    // reading that shows what it carries: black glyphs on a black field would be a picture of nothing.
    let image: RgbaImage =
      decode_uncompressed(&dds(0x2, 8, [0, 0, 0, 0xff], &[0, 0x80, 0xff, 0x40]), 0).expect("expect A8 to expand");

    assert_eq!(
      pixels(&image),
      vec![
        [255, 255, 255, 0],
        [255, 255, 255, 0x80],
        [255, 255, 255, 0xff],
        [255, 255, 255, 0x40]
      ]
    );
  }

  #[test]
  fn expands_luminance_across_the_colour_channels() {
    let image: RgbaImage =
      decode_uncompressed(&dds(0x20000, 8, [0xff, 0, 0, 0], &[0, 0x40, 0x80, 0xff]), 0).expect("expect L8 to expand");

    assert_eq!(
      pixels(&image),
      vec![
        [0, 0, 0, 255],
        [0x40, 0x40, 0x40, 255],
        [0x80, 0x80, 0x80, 255],
        [255, 255, 255, 255]
      ]
    );
  }

  #[test]
  fn expands_alpha_luminance_into_grey_with_coverage() {
    // Sixteen bits, luminance in the low byte and alpha in the high one.
    let payload: Vec<u8> = vec![0x00, 0xff, 0x80, 0x80, 0xff, 0x00, 0x40, 0xc0];
    let image: RgbaImage = decode_uncompressed(&dds(0x20001, 16, [0x00ff, 0, 0, 0xff00], &payload), 0)
      .expect("expect alpha-luminance to expand");

    assert_eq!(
      pixels(&image),
      vec![
        [0, 0, 0, 255],
        [0x80, 0x80, 0x80, 0x80],
        [255, 255, 255, 0],
        [0x40, 0x40, 0x40, 0xc0]
      ]
    );
  }

  #[test]
  fn scales_five_and_six_bit_channels_across_the_whole_range() {
    // The point of scaling rather than shifting: five bits of white are 31, and shifting them left by three gives 248,
    // a grey that makes every light surface slightly dirty.
    let white: u16 = 0xffff;
    let red: u16 = 0xf800;
    let payload: Vec<u8> = [white, red, 0x07e0, 0x001f]
      .iter()
      .flat_map(|it| it.to_le_bytes())
      .collect();
    let image: RgbaImage =
      decode_uncompressed(&dds(0x40, 16, [0xf800, 0x07e0, 0x001f, 0], &payload), 0).expect("expect R5G6B5 to expand");

    assert_eq!(
      pixels(&image),
      vec![
        [255, 255, 255, 255],
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255]
      ]
    );
  }

  #[test]
  fn reads_a_thirty_two_bit_layout_with_no_alpha_mask_as_opaque() {
    // `X8R8G8B8` carries eight unused bits where alpha would be, and reading them as coverage would make every pixel
    // of these files transparent.
    let payload: Vec<u8> = vec![
      0x00, 0x00, 0xff, 0x7f, // blue, with rubbish in the unused byte
      0x00, 0xff, 0x00, 0x00, // green
      0xff, 0x00, 0x00, 0xff, // red
      0xff, 0xff, 0xff, 0x00, // white
    ];
    let image: RgbaImage = decode_uncompressed(&dds(0x40, 32, [0x00ff0000, 0x0000ff00, 0x000000ff, 0], &payload), 0)
      .expect("expect X8R8G8B8 to expand");

    assert_eq!(
      pixels(&image),
      vec![
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 255, 255]
      ]
    );
  }

  #[test]
  fn refuses_a_payload_shorter_than_the_level_it_describes() {
    let error = decode_uncompressed(&dds(0x2, 8, [0, 0, 0, 0xff], &[0, 0x80]), 0).expect_err("expect a short payload");

    assert!(error.to_string().contains("needs 4"), "got {error}");
  }
}
