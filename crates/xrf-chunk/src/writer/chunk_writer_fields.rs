use std::io::Write;

use xrf_error::{XrfError, XrfResult};

use crate::bytes::xray_bytes::XRayBytes;
use crate::bytes::xray_write::XRayWrite;
use crate::field::chunk_field_writer::ChunkFieldWriter;
use crate::writer::chunk_writer::ChunkWriter;

/// The primitives as bytes, as a `NET_Packet` without an ini stream writes them.
impl ChunkFieldWriter for ChunkWriter {
  #[inline]
  fn is_field_form(&self) -> bool {
    false
  }

  #[inline]
  fn write_u8(&mut self, value: u8) -> XrfResult {
    Ok(XRayWrite::write_u8(self, value)?)
  }

  #[inline]
  fn write_u16(&mut self, value: u16) -> XrfResult {
    Ok(XRayWrite::write_u16(self, value)?)
  }

  #[inline]
  fn write_u32(&mut self, value: u32) -> XrfResult {
    Ok(XRayWrite::write_u32(self, value)?)
  }

  #[inline]
  fn write_u64(&mut self, value: u64) -> XrfResult {
    Ok(XRayWrite::write_u64(self, value)?)
  }

  #[inline]
  fn write_f32(&mut self, value: f32) -> XrfResult {
    Ok(XRayWrite::write_f32(self, value)?)
  }

  #[inline]
  fn write_vector3(&mut self, value: [f32; 3]) -> XrfResult {
    for component in value {
      XRayWrite::write_f32(self, component)?;
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
  fn patch_u16(&mut self, at: usize, value: u16) -> XrfResult {
    let written: usize = self.bytes_written();
    let bytes: &mut [u8] = self
      .buffer
      .get_mut(at..at + size_of::<u16>())
      .ok_or_else(|| XrfError::new_invalid_error(format!("Cannot patch a u16 at {at}, {written} bytes are written")))?;

    XRayBytes::write_u16(bytes, value);

    Ok(())
  }
}
