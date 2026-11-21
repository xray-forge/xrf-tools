use std::io::{Read, Result};

/// Reads a number of each width the X-Ray formats store, little-endian as every one of them is written.
///
/// Implemented for every reader. A [`crate::ChunkReader`] is one, and is also a [`crate::ChunkFieldReader`], whose
/// methods share these names, so a file using both traits calls one through its trait name.
pub trait XRayRead: Read {
  #[inline]
  fn read_u8(&mut self) -> Result<u8> {
    Ok(u8::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_i8(&mut self) -> Result<i8> {
    Ok(i8::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_u16(&mut self) -> Result<u16> {
    Ok(u16::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_i16(&mut self) -> Result<i16> {
    Ok(i16::from_le_bytes(read_array(self)?))
  }

  /// Three bytes as the low part of a `u32`, as the AI map packs a node's links.
  #[inline]
  fn read_u24(&mut self) -> Result<u32> {
    let [low, middle, high]: [u8; 3] = read_array(self)?;

    Ok(u32::from_le_bytes([low, middle, high, 0]))
  }

  #[inline]
  fn read_u32(&mut self) -> Result<u32> {
    Ok(u32::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_i32(&mut self) -> Result<i32> {
    Ok(i32::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_u64(&mut self) -> Result<u64> {
    Ok(u64::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_i64(&mut self) -> Result<i64> {
    Ok(i64::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_u128(&mut self) -> Result<u128> {
    Ok(u128::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_f32(&mut self) -> Result<f32> {
    Ok(f32::from_le_bytes(read_array(self)?))
  }

  #[inline]
  fn read_f64(&mut self) -> Result<f64> {
    Ok(f64::from_le_bytes(read_array(self)?))
  }
}

impl<R: Read + ?Sized> XRayRead for R {}

#[inline]
fn read_array<const N: usize, R: Read + ?Sized>(reader: &mut R) -> Result<[u8; N]> {
  let mut bytes: [u8; N] = [0; N];

  reader.read_exact(&mut bytes)?;

  Ok(bytes)
}
