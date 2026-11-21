/// Numbers read from and written into the front of a byte slice, little-endian as every X-Ray format is written.
///
/// For a record decoded from bytes already in hand; a stream reads through [`crate::XRayRead`] and
/// [`crate::XRayWrite`] instead.
///
/// Every function panics when the slice is shorter than the number it reads or writes.
pub struct XRayBytes;

impl XRayBytes {
  #[inline]
  pub fn read_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(front(bytes))
  }

  #[inline]
  pub fn read_i16(bytes: &[u8]) -> i16 {
    i16::from_le_bytes(front(bytes))
  }

  /// Three bytes as the low part of a `u32`.
  #[inline]
  pub fn read_u24(bytes: &[u8]) -> u32 {
    let [low, middle, high]: [u8; 3] = front(bytes);

    u32::from_le_bytes([low, middle, high, 0])
  }

  #[inline]
  pub fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(front(bytes))
  }

  #[inline]
  pub fn read_f32(bytes: &[u8]) -> f32 {
    f32::from_le_bytes(front(bytes))
  }

  /// `size` bytes, one to eight, as an unsigned number.
  ///
  /// # Panics
  ///
  /// Panics when `size` is not between one and eight, or the slice is shorter.
  #[inline]
  pub fn read_uint(bytes: &[u8], size: usize) -> u64 {
    assert!(
      (1..=8).contains(&size),
      "a {size}-byte number is not one to eight bytes"
    );

    let mut wide: [u8; 8] = [0; 8];

    wide[..size].copy_from_slice(&bytes[..size]);

    u64::from_le_bytes(wide)
  }

  #[inline]
  pub fn write_u16(bytes: &mut [u8], value: u16) {
    bytes[..2].copy_from_slice(&value.to_le_bytes());
  }

  #[inline]
  pub fn write_i16(bytes: &mut [u8], value: i16) {
    bytes[..2].copy_from_slice(&value.to_le_bytes());
  }

  /// The low three bytes of `value`.
  ///
  /// # Panics
  ///
  /// Panics when `value` does not fit in three bytes, or the slice is shorter.
  #[inline]
  pub fn write_u24(bytes: &mut [u8], value: u32) {
    assert!(value < 1 << 24, "{value} does not fit in three bytes");

    bytes[..3].copy_from_slice(&value.to_le_bytes()[..3]);
  }

  #[inline]
  pub fn write_u32(bytes: &mut [u8], value: u32) {
    bytes[..4].copy_from_slice(&value.to_le_bytes());
  }

  #[inline]
  pub fn write_f32(bytes: &mut [u8], value: f32) {
    bytes[..4].copy_from_slice(&value.to_le_bytes());
  }

  /// `value` in `size` bytes, one to eight.
  ///
  /// # Panics
  ///
  /// Panics when `size` is not between one and eight, `value` does not fit in it, or the slice is shorter.
  #[inline]
  pub fn write_uint(bytes: &mut [u8], value: u64, size: usize) {
    assert!(
      (1..=8).contains(&size),
      "a {size}-byte number is not one to eight bytes"
    );
    assert!(
      size == 8 || value < 1 << (size * 8),
      "{value} does not fit in {size} bytes"
    );

    bytes[..size].copy_from_slice(&value.to_le_bytes()[..size]);
  }
}

#[inline]
fn front<const N: usize>(bytes: &[u8]) -> [u8; N] {
  let mut array: [u8; N] = [0; N];

  array.copy_from_slice(&bytes[..N]);

  array
}

#[cfg(test)]
mod tests {
  use crate::bytes::xray_bytes::XRayBytes;

  #[test]
  fn reads_and_writes_the_front_of_a_slice_little_endian() {
    let mut bytes: [u8; 8] = [0xAA; 8];

    XRayBytes::write_u16(&mut bytes, 0x0201);
    assert_eq!(bytes, [0x01, 0x02, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA]);
    assert_eq!(XRayBytes::read_u16(&bytes), 0x0201);

    XRayBytes::write_i16(&mut bytes, -2);
    assert_eq!(XRayBytes::read_i16(&bytes), -2);

    XRayBytes::write_u24(&mut bytes, 0x03_0201);
    assert_eq!(&bytes[..4], [0x01, 0x02, 0x03, 0xAA]);
    assert_eq!(XRayBytes::read_u24(&bytes), 0x03_0201);

    XRayBytes::write_u32(&mut bytes, 0x0403_0201);
    assert_eq!(XRayBytes::read_u32(&bytes), 0x0403_0201);

    XRayBytes::write_f32(&mut bytes, 2.5);
    assert_eq!(XRayBytes::read_f32(&bytes).to_bits(), 2.5_f32.to_bits());
  }

  #[test]
  fn reads_and_writes_a_number_of_any_width_up_to_eight_bytes() {
    let mut bytes: [u8; 8] = [0xAA; 8];

    XRayBytes::write_uint(&mut bytes, 0x0201, 2);
    assert_eq!(&bytes[..3], [0x01, 0x02, 0xAA]);
    assert_eq!(XRayBytes::read_uint(&bytes, 2), 0x0201);

    XRayBytes::write_uint(&mut bytes, u64::MAX, 8);
    assert_eq!(XRayBytes::read_uint(&bytes, 8), u64::MAX);
  }

  #[test]
  #[should_panic(expected = "does not fit in 1 bytes")]
  fn refuses_a_number_wider_than_its_size() {
    XRayBytes::write_uint(&mut [0; 2], 0x100, 1);
  }

  #[test]
  #[should_panic]
  fn refuses_a_slice_shorter_than_the_number() {
    let _ = XRayBytes::read_u32(&[0, 0, 0]);
  }
}
