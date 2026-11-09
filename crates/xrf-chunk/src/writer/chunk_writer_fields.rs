use std::io::Write;

use byteorder::{ByteOrder, WriteBytesExt};
use xrf_error::{XrfError, XrfResult};

use crate::field::chunk_field_writer::ChunkFieldWriter;
use crate::writer::chunk_writer::ChunkWriter;

/// The primitives as bytes, which is what a `NET_Packet` without an ini stream writes.
impl ChunkFieldWriter for ChunkWriter {
  #[inline]
  fn is_field_form(&self) -> bool {
    false
  }

  #[inline]
  fn write_u8(&mut self, value: u8) -> XrfResult {
    Ok(WriteBytesExt::write_u8(self, value)?)
  }

  #[inline]
  fn write_u16<T: ByteOrder>(&mut self, value: u16) -> XrfResult {
    Ok(WriteBytesExt::write_u16::<T>(self, value)?)
  }

  #[inline]
  fn write_u32<T: ByteOrder>(&mut self, value: u32) -> XrfResult {
    Ok(WriteBytesExt::write_u32::<T>(self, value)?)
  }

  #[inline]
  fn write_u64<T: ByteOrder>(&mut self, value: u64) -> XrfResult {
    Ok(WriteBytesExt::write_u64::<T>(self, value)?)
  }

  #[inline]
  fn write_f32<T: ByteOrder>(&mut self, value: f32) -> XrfResult {
    Ok(WriteBytesExt::write_f32::<T>(self, value)?)
  }

  #[inline]
  fn write_vector3<T: ByteOrder>(&mut self, value: [f32; 3]) -> XrfResult {
    for component in value {
      WriteBytesExt::write_f32::<T>(self, component)?;
    }

    Ok(())
  }

  #[inline]
  fn write_w1251_string(&mut self, value: &str) -> XrfResult {
    Self::write_w1251_string(self, value)?;

    Ok(())
  }

  #[inline]
  fn write_bytes(&mut self, bytes: &[u8]) -> XrfResult {
    Ok(self.write_all(bytes)?)
  }

  #[inline]
  fn get_position(&self) -> usize {
    self.bytes_written()
  }

  #[inline]
  fn patch_u16<T: ByteOrder>(&mut self, at: usize, value: u16) -> XrfResult {
    let written: usize = self.bytes_written();
    let bytes: &mut [u8] = self
      .buffer
      .get_mut(at..at + size_of::<u16>())
      .ok_or_else(|| XrfError::new_invalid_error(format!("Cannot patch a u16 at {at}, {written} bytes are written")))?;

    T::write_u16(bytes, value);

    Ok(())
  }
}
