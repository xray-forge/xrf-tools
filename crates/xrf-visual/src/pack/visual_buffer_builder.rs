use crate::data::visual_section::VisualSection;

/// Accumulates a visual's attribute buffers into one blob and reports where each landed.
///
/// Owning offset accounting here is what keeps it out of the packer: an attribute is added by pushing
/// its values, and its byte range comes back rather than being computed alongside. Adding a new
/// attribute therefore cannot mis-align or overlap an existing one.
///
/// Values are written little endian because the consumer reads them through typed array views, which
/// use the host's byte order, and every supported target is little endian.
#[derive(Debug, Default)]
pub struct VisualBufferBuilder {
  buffer: Vec<u8>,
}

impl VisualBufferBuilder {
  /// Every section starts on this boundary, because a `Float32Array` view refuses a byte offset that
  /// is not a multiple of four and a `Uint16Array` view one that is not even.
  const ALIGNMENT: usize = 4;

  pub fn new() -> Self {
    Self::default()
  }

  /// Appends `f32` values as little-endian bytes and returns their aligned byte range.
  ///
  /// The returned offset is suitable for a `Float32Array` view; padding before the section is
  /// included in the buffer but not in its range.
  pub fn push_f32_section(&mut self, values: &[f32]) -> VisualSection {
    self.push_encoded(values, f32::to_le_bytes)
  }

  /// Appends `u16` values as little-endian bytes and returns their aligned byte range.
  ///
  /// The returned offset is suitable for a `Uint16Array` view; padding before the section is
  /// included in the buffer but not in its range.
  pub fn push_u16_section(&mut self, values: &[u16]) -> VisualSection {
    self.push_encoded(values, u16::to_le_bytes)
  }

  /// Appends `u32` values as little-endian bytes and returns their aligned byte range.
  pub fn push_u32_section(&mut self, values: &[u32]) -> VisualSection {
    self.push_encoded(values, u32::to_le_bytes)
  }

  /// Returns the total packed buffer length, including alignment padding.
  pub fn length(&self) -> u32 {
    Self::usize_to_u32(self.buffer.len())
  }

  /// Returns the packed bytes, including alignment padding between sections.
  pub fn into_buffer(self) -> Vec<u8> {
    self.buffer
  }

  /// Appends one section by writing each value's bytes into room made for all of them at once.
  fn push_encoded<const N: usize, V: Copy>(&mut self, values: &[V], encode: impl Fn(V) -> [u8; N]) -> VisualSection {
    self.push_section(size_of_val(values), |buffer| {
      let at: usize = buffer.len();

      buffer.resize(at + values.len() * N, 0);

      for (slot, value) in buffer[at..].as_chunks_mut::<N>().0.iter_mut().zip(values) {
        *slot = encode(*value);
      }
    })
  }

  fn push_section(&mut self, byte_length: usize, write: impl FnOnce(&mut Vec<u8>)) -> VisualSection {
    self.align();

    let byte_offset: usize = self.buffer.len();

    write(&mut self.buffer);

    debug_assert_eq!(
      self.buffer.len() - byte_offset,
      byte_length,
      "a section must write exactly the length it reported"
    );

    VisualSection {
      byte_offset: Self::usize_to_u32(byte_offset),
      byte_length: Self::usize_to_u32(byte_length),
    }
  }

  fn align(&mut self) {
    let remainder: usize = self.buffer.len() % Self::ALIGNMENT;

    if remainder != 0 {
      self.buffer.resize(self.buffer.len() + Self::ALIGNMENT - remainder, 0);
    }
  }

  /// Offsets are `u32` on the wire. A visual's packed geometry is megabytes, so the ceiling is four
  /// thousand times the largest file measured across the reference trees.
  fn usize_to_u32(value: usize) -> u32 {
    debug_assert!(value <= u32::MAX as usize, "a geometry buffer must fit a u32 offset");

    value as u32
  }
}
