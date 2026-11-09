use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};

use crate::field::chunk_field::ChunkField;
use crate::field::chunk_field_writer::ChunkFieldWriter;

/// Records each value written as a field of its kind, as a `NET_Packet` with an ini stream attached saves one per
/// primitive (the Level Editor's `SIniFileStream`).
///
/// Its position counts the bytes the values take, as the packet's own count does, so a size or save marker taken from
/// it is the one the bytes would hold. A patch is a seek over the bytes alone and leaves the fields as written. A raw
/// write makes no field: one of no bytes writes nothing, any other is refused, as `NET_Packet::w` asserts there.
#[derive(Debug, Default)]
pub struct ChunkFieldRecorder {
  fields: Vec<ChunkField>,
  position: usize,
}

impl ChunkFieldRecorder {
  pub fn new() -> Self {
    Self::default()
  }

  /// The fields written, in order.
  pub fn into_fields(self) -> Vec<ChunkField> {
    self.fields
  }

  fn record(&mut self, field: ChunkField) -> XrfResult {
    self.position += field.measure_bytes()?;
    self.fields.push(field);

    Ok(())
  }
}

impl ChunkFieldWriter for ChunkFieldRecorder {
  fn is_field_form(&self) -> bool {
    true
  }

  fn write_u8(&mut self, value: u8) -> XrfResult {
    self.record(ChunkField::U8(value))
  }

  fn write_u16<T: ByteOrder>(&mut self, value: u16) -> XrfResult {
    self.record(ChunkField::U16(value))
  }

  fn write_u32<T: ByteOrder>(&mut self, value: u32) -> XrfResult {
    self.record(ChunkField::U32(value))
  }

  fn write_u64<T: ByteOrder>(&mut self, value: u64) -> XrfResult {
    self.record(ChunkField::U64(value))
  }

  fn write_f32<T: ByteOrder>(&mut self, value: f32) -> XrfResult {
    self.record(ChunkField::F32(value))
  }

  fn write_vector3<T: ByteOrder>(&mut self, value: [f32; 3]) -> XrfResult {
    self.record(ChunkField::Vector3(value))
  }

  fn write_w1251_string(&mut self, value: &str) -> XrfResult {
    self.record(ChunkField::String(value.to_owned()))
  }

  fn write_bytes(&mut self, bytes: &[u8]) -> XrfResult {
    if bytes.is_empty() {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "Cannot record {} raw bytes, which make no field",
      bytes.len()
    )))
  }

  fn get_position(&self) -> usize {
    self.position
  }

  fn patch_u16<T: ByteOrder>(&mut self, _: usize, _: u16) -> XrfResult {
    Ok(())
  }
}
