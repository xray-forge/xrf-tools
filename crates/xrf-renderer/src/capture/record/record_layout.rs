/// How a recording's frames lie in the buffers they are copied into: pixels across and down, and the bytes a row of a
/// copy takes, aligned as copies need.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RecordLayout {
  pub(crate) size: (u32, u32),
  pub(crate) row: u32,
}

impl RecordLayout {
  /// Frames `size` pixels across and down, four bytes each.
  pub(crate) fn new(size: (u32, u32)) -> Self {
    Self {
      size,
      row: (size.0 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
    }
  }

  /// Bytes a frame's buffer takes.
  pub(crate) fn get_buffer_size(&self) -> u64 {
    u64::from(self.row) * u64::from(self.size.1)
  }
}
