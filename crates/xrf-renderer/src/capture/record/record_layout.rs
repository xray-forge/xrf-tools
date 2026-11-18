/// A recording frame's buffer layout: size in pixels and the copy-aligned bytes per row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RecordLayout {
  pub(crate) size: (u32, u32),
  pub(crate) row: u32,
}

impl RecordLayout {
  /// The layout of `size`-pixel frames at four bytes a pixel.
  pub(crate) fn new(size: (u32, u32)) -> Self {
    Self {
      size,
      row: (size.0 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
    }
  }

  /// Bytes of one frame's buffer.
  pub(crate) fn get_buffer_size(&self) -> u64 {
    u64::from(self.row) * u64::from(self.size.1)
  }
}
