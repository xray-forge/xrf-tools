use std::io::{Result, Write};

/// Writes a number of each width the X-Ray formats store, little-endian as every one of them is written.
///
/// Implemented for every writer. A [`crate::ChunkWriter`] is one, and is also a [`crate::ChunkFieldWriter`], whose
/// methods share these names, so a file using both traits calls one through its trait name.
pub trait XRayWrite: Write {
  #[inline]
  fn write_u8(&mut self, value: u8) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_i8(&mut self, value: i8) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_u16(&mut self, value: u16) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_i16(&mut self, value: i16) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  /// The low three bytes of `value`, as the AI map packs a node's links.
  ///
  /// # Panics
  ///
  /// Panics when `value` does not fit in three bytes.
  #[inline]
  fn write_u24(&mut self, value: u32) -> Result<()> {
    assert!(value < 1 << 24, "{value} does not fit in three bytes");

    self.write_all(&value.to_le_bytes()[..3])
  }

  #[inline]
  fn write_u32(&mut self, value: u32) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_i32(&mut self, value: i32) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_u64(&mut self, value: u64) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_i64(&mut self, value: i64) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_u128(&mut self, value: u128) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_f32(&mut self, value: f32) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }

  #[inline]
  fn write_f64(&mut self, value: f64) -> Result<()> {
    self.write_all(&value.to_le_bytes())
  }
}

impl<W: Write + ?Sized> XRayWrite for W {}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use crate::{XRayRead, XRayWrite};

  #[test]
  fn writes_every_width_little_endian_and_reads_it_back() -> std::io::Result<()> {
    let mut bytes: Vec<u8> = Vec::new();

    bytes.write_u8(0x01)?;
    bytes.write_i8(-2)?;
    bytes.write_u16(0x0302)?;
    bytes.write_i16(-3)?;
    bytes.write_u24(0x06_0504)?;
    bytes.write_u32(0x0A09_0807)?;
    bytes.write_i32(-4)?;
    bytes.write_u64(0x1211_100F_0E0D_0C0B)?;
    bytes.write_i64(-5)?;
    bytes.write_u128(u128::MAX - 1)?;
    bytes.write_f32(1.0)?;
    bytes.write_f64(-0.5)?;

    assert_eq!(
      &bytes[..12],
      [0x01, 0xFE, 0x02, 0x03, 0xFD, 0xFF, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09]
    );

    let mut reader: Cursor<Vec<u8>> = Cursor::new(bytes);

    assert_eq!(reader.read_u8()?, 0x01);
    assert_eq!(reader.read_i8()?, -2);
    assert_eq!(reader.read_u16()?, 0x0302);
    assert_eq!(reader.read_i16()?, -3);
    assert_eq!(reader.read_u24()?, 0x06_0504);
    assert_eq!(reader.read_u32()?, 0x0A09_0807);
    assert_eq!(reader.read_i32()?, -4);
    assert_eq!(reader.read_u64()?, 0x1211_100F_0E0D_0C0B);
    assert_eq!(reader.read_i64()?, -5);
    assert_eq!(reader.read_u128()?, u128::MAX - 1);
    assert_eq!(reader.read_f32()?.to_bits(), 1.0_f32.to_bits());
    assert_eq!(reader.read_f64()?.to_bits(), (-0.5_f64).to_bits());
    assert!(reader.read_u8().is_err(), "nothing is left");

    Ok(())
  }

  #[test]
  #[should_panic(expected = "does not fit in three bytes")]
  fn refuses_a_three_byte_value_that_does_not_fit() {
    let _ = Vec::new().write_u24(1 << 24);
  }
}
