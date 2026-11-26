use image_dds::ImageFormat;

/// How a block-compressed layout stores its texels: four by four a block, and the bytes each block takes. The one rule
/// the stripe decoder and the encoder both size their pieces by.
pub(crate) struct DdsBlockLayout;

impl DdsBlockLayout {
  /// Pixels along one edge of a block.
  pub(crate) const SIDE: u32 = 4;

  /// Bytes a block of a block-compressed layout takes, or `None` for any other layout.
  pub(crate) fn block_bytes(format: ImageFormat) -> Option<usize> {
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

  /// Bytes one row of blocks takes across a level `width` pixels wide.
  pub(crate) fn block_row_bytes(width: u32, block_bytes: usize) -> usize {
    width.div_ceil(Self::SIDE) as usize * block_bytes
  }
}
