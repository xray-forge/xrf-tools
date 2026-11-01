/// Where a draw's indirect arguments start: their buffer, and the byte offset in it.
#[derive(Clone, Copy)]
pub struct IndirectArgs<'a> {
  pub buffer: &'a wgpu::Buffer,
  pub offset: u64,
}

impl<'a> IndirectArgs<'a> {
  /// A buffer's arguments from its start.
  pub fn of(buffer: &'a wgpu::Buffer) -> Self {
    Self { buffer, offset: 0 }
  }
}
