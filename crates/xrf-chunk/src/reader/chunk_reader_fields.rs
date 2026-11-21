use xrf_error::XrfResult;

use crate::bytes::xray_read::XRayRead;
use crate::field::chunk_field_reader::ChunkFieldReader;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_data_source::ChunkDataSource;

/// The primitives as bytes, as a `NET_Packet` without an ini stream reads them.
impl<D: ChunkDataSource> ChunkFieldReader for ChunkReader<D> {
  #[inline]
  fn is_field_form(&self) -> bool {
    false
  }

  #[inline]
  fn read_u8(&mut self) -> XrfResult<u8> {
    Ok(XRayRead::read_u8(self)?)
  }

  #[inline]
  fn read_u16(&mut self) -> XrfResult<u16> {
    Ok(XRayRead::read_u16(self)?)
  }

  #[inline]
  fn read_u32(&mut self) -> XrfResult<u32> {
    Ok(XRayRead::read_u32(self)?)
  }

  #[inline]
  fn read_u64(&mut self) -> XrfResult<u64> {
    Ok(XRayRead::read_u64(self)?)
  }

  #[inline]
  fn read_f32(&mut self) -> XrfResult<f32> {
    Ok(XRayRead::read_f32(self)?)
  }

  #[inline]
  fn read_vector3(&mut self) -> XrfResult<[f32; 3]> {
    Ok([
      XRayRead::read_f32(self)?,
      XRayRead::read_f32(self)?,
      XRayRead::read_f32(self)?,
    ])
  }

  #[inline]
  fn read_w1251_string(&mut self) -> XrfResult<String> {
    Self::read_w1251_string(self)
  }

  #[inline]
  fn read_bytes(&mut self, count: usize) -> XrfResult<Vec<u8>> {
    Self::read_bytes(self, count)
  }

  #[inline]
  fn get_position(&self) -> usize {
    usize::try_from(self.read_bytes_len()).expect("bytes read from a chunk fit in memory")
  }

  #[inline]
  fn is_ended(&self) -> bool {
    Self::is_ended(self)
  }

  #[inline]
  fn new_bounded_vec<V>(&self, count: u64, min_record_size: u64, what: &str) -> XrfResult<Vec<V>> {
    Self::new_bounded_vec(self, count, min_record_size, what)
  }
}
