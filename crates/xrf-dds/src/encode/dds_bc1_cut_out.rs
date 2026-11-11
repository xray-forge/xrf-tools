/// BC1's one bit of alpha, which the block encoder leaves out, written back into the blocks that need it: the format the
/// SDK calls `tfADXT1`. A block whose two endpoints stand in ascending order decodes in three colours, its fourth index
/// transparent black; so a block covering a pixel of alpha below [`Self::THRESHOLD`] is rewritten that way, its endpoints
/// taken from the opaque encoding, ordered, and each opaque pixel given the nearest of the three colours.
pub(crate) struct DdsBc1CutOut;

impl DdsBc1CutOut {
  /// Alpha below which a pixel is cut out, as Direct3D's own DXT1 conversion rounds it.
  pub(crate) const THRESHOLD: u8 = 128;

  /// Bytes one block occupies: two endpoints and sixteen two-bit indices.
  const BLOCK_BYTES: usize = 8;

  /// The index a three-colour block draws transparent.
  const TRANSPARENT: u32 = 3;

  /// Rewrites each block of `blocks`, an image `width` by `height`'s BC1 blocks row by row, that covers a cut-out pixel
  /// of `pixels`, its RGBA row by row. A block past the image's edge takes its pixels as cut out where none stands.
  pub(crate) fn apply(blocks: &mut [u8], pixels: &[u8], width: u32, height: u32) {
    let blocks_across: usize = width.div_ceil(4) as usize;

    for (index, block) in blocks.as_chunks_mut::<{ Self::BLOCK_BYTES }>().0.iter_mut().enumerate() {
      let (block_x, block_y) = ((index % blocks_across) as u32 * 4, (index / blocks_across) as u32 * 4);
      // Each of the block's sixteen pixels, `None` past the image's edge.
      let texels: [Option<[u8; 4]>; 16] = std::array::from_fn(|texel| {
        let (x, y) = (block_x + texel as u32 % 4, block_y + texel as u32 / 4);

        (x < width && y < height).then(|| {
          let start: usize = (y as usize * width as usize + x as usize) * 4;

          [pixels[start], pixels[start + 1], pixels[start + 2], pixels[start + 3]]
        })
      });

      if texels.iter().flatten().any(|texel| texel[3] < Self::THRESHOLD) {
        Self::rewrite(block, &texels);
      }
    }
  }

  /// `block` in three colours: its endpoints ordered, each opaque pixel the nearest of them and their midpoint, each
  /// cut-out pixel transparent.
  fn rewrite(block: &mut [u8; Self::BLOCK_BYTES], texels: &[Option<[u8; 4]>; 16]) {
    let first: u16 = u16::from_le_bytes([block[0], block[1]]);
    let second: u16 = u16::from_le_bytes([block[2], block[3]]);
    let (low, high) = (first.min(second), first.max(second));
    let (low_rgb, high_rgb) = (Self::expand(low), Self::expand(high));
    let palette: [[i32; 3]; 3] = [
      low_rgb,
      high_rgb,
      std::array::from_fn(|channel| i32::midpoint(low_rgb[channel], high_rgb[channel])),
    ];
    let indices: u32 = texels.iter().enumerate().fold(0, |indices, (texel, pixel)| {
      let index: u32 = match pixel {
        Some(pixel) if pixel[3] >= Self::THRESHOLD => Self::nearest(&palette, pixel),
        _ => Self::TRANSPARENT,
      };

      indices | (index << (texel * 2))
    });

    block[0..2].copy_from_slice(&low.to_le_bytes());
    block[2..4].copy_from_slice(&high.to_le_bytes());
    block[4..8].copy_from_slice(&indices.to_le_bytes());
  }

  /// The index of the colour of `palette` nearest `pixel`.
  fn nearest(palette: &[[i32; 3]; 3], pixel: &[u8; 4]) -> u32 {
    (0u32..)
      .zip(palette)
      .min_by_key(|(_, colour)| {
        (0..3)
          .map(|channel| (colour[channel] - i32::from(pixel[channel])).pow(2))
          .sum::<i32>()
      })
      .map_or(0, |(index, _)| index)
  }

  /// An endpoint's five, six and five bits widened to eight each, as a decoder widens them.
  fn expand(colour: u16) -> [i32; 3] {
    let (red, green, blue) = (
      i32::from(colour >> 11),
      i32::from((colour >> 5) & 0x3F),
      i32::from(colour & 0x1F),
    );

    [
      (red << 3) | (red >> 2),
      (green << 2) | (green >> 4),
      (blue << 3) | (blue >> 2),
    ]
  }
}

#[cfg(test)]
mod tests {
  use super::DdsBc1CutOut;

  /// One block, opaque red and green endpoints in descending order, its indices all 0.
  fn opaque_block() -> [u8; 8] {
    let (red, green): (u16, u16) = (0xF800, 0x07E0);
    let mut block: [u8; 8] = [0; 8];

    block[0..2].copy_from_slice(&red.to_le_bytes());
    block[2..4].copy_from_slice(&green.to_le_bytes());
    block
  }

  #[test]
  fn rewrites_a_block_covering_a_cut_out_pixel_in_three_colours() {
    let mut pixels: Vec<u8> = [255, 0, 0, 255].repeat(16);

    // The second pixel cut out, the third green.
    pixels[4..8].copy_from_slice(&[0, 0, 0, 0]);
    pixels[8..12].copy_from_slice(&[0, 255, 0, 255]);

    let mut block: [u8; 8] = opaque_block();

    DdsBc1CutOut::apply(&mut block, &pixels, 4, 4);

    let (first, second) = (
      u16::from_le_bytes([block[0], block[1]]),
      u16::from_le_bytes([block[2], block[3]]),
    );
    let indices: u32 = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);

    assert!(first <= second, "ascending endpoints select three colours");
    assert_eq!(indices & 0b11, 1, "red, now the second endpoint");
    assert_eq!((indices >> 2) & 0b11, 3, "transparent");
    assert_eq!((indices >> 4) & 0b11, 0, "green, now the first endpoint");
  }

  #[test]
  fn leaves_an_opaque_block_as_encoded() {
    let mut block: [u8; 8] = opaque_block();

    DdsBc1CutOut::apply(&mut block, &[255, 0, 0, 255].repeat(16), 4, 4);

    assert_eq!(block, opaque_block());
  }
}
