use xrf_error::XrfResult;
use xrf_utils::narrow_format_size;

use crate::read_write::chunk_field_read_write::ChunkFieldReadWrite;
use crate::read_write::chunk_field_read_write_list::ChunkFieldReadWriteList;
use crate::read_write::chunk_field_read_write_optional::ChunkFieldReadWriteOptional;

/// What a [`ChunkFieldReadWrite`] value writes itself to, the engine's `NET_Packet` primitives (`NET_utils.h`): bytes
/// through a [`crate::ChunkWriter`], one field per primitive through a [`crate::ChunkFieldRecorder`].
///
/// Method names match [`crate::ChunkWriter`]'s, so a value's code reads the same over either.
pub trait ChunkFieldWriter {
  /// Whether values become fields rather than bytes, `NET_Packet::inistream != nullptr`. Check it where the engine
  /// writes a value differently for an ini stream.
  fn is_field_form(&self) -> bool;

  fn write_u8(&mut self, value: u8) -> XrfResult;

  fn write_u16(&mut self, value: u16) -> XrfResult;

  fn write_u32(&mut self, value: u32) -> XrfResult;

  fn write_u64(&mut self, value: u64) -> XrfResult;

  fn write_f32(&mut self, value: f32) -> XrfResult;

  /// Three floats as one value, `w_vec3`.
  fn write_vector3(&mut self, value: [f32; 3]) -> XrfResult;

  /// A null-terminated Windows-1251 string, `w_stringZ`.
  fn write_w1251_string(&mut self, value: &str) -> XrfResult;

  /// Bytes as they are, `w`, which makes no field.
  fn write_bytes(&mut self, bytes: &[u8]) -> XrfResult;

  /// The bytes written so far, `w_tell`.
  fn get_position(&self) -> usize;

  /// Overwrites the `u16` at `at` in the bytes, `w_seek`: a field recorded there keeps the value it was written with.
  fn patch_u16(&mut self, at: usize, value: u16) -> XrfResult;

  fn write_xr<V: ChunkFieldReadWrite>(&mut self, value: &V) -> XrfResult
  where
    Self: Sized,
  {
    value.write_fields::<Self>(self)
  }

  fn write_xr_list<V: ChunkFieldReadWriteList>(&mut self, list: &[V]) -> XrfResult
  where
    Self: Sized,
  {
    V::write_field_list::<Self>(self, list)
  }

  fn write_xr_optional<V: ChunkFieldReadWriteOptional>(&mut self, value: Option<&V>) -> XrfResult
  where
    Self: Sized,
  {
    V::write_field_optional::<Self>(self, value)
  }

  /// An `xr_vector<u16>` as `save_data` writes it: a `u32` count, then each.
  fn write_u16_vector(&mut self, values: &[u16]) -> XrfResult {
    self.write_u32(narrow_format_size(values.len(), "u16 vector length")?)?;

    for value in values {
      self.write_u16(*value)?;
    }

    Ok(())
  }
}
