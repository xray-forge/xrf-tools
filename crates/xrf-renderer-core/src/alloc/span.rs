use offset_allocator::Allocation;

/// A range of a span buffer's elements, held until freed; not copied, so it is freed once.
pub struct Span {
  /// Its first element in the buffer.
  pub(crate) offset: u32,
  /// Elements asked for; the allocator may reserve a few more.
  pub(crate) count: u32,
  pub(crate) region: u32,
  pub(crate) allocation: Allocation,
}

impl Span {
  /// Its first element in the buffer, which a shader indexes from.
  pub fn get_offset(&self) -> u32 {
    self.offset
  }

  pub fn get_count(&self) -> u32 {
    self.count
  }
}

impl std::fmt::Debug for Span {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("Span")
      .field("offset", &self.offset)
      .field("count", &self.count)
      .field("region", &self.region)
      .finish()
  }
}
