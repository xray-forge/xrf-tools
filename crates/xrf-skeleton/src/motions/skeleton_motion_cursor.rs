use byteorder::{ByteOrder, ReadBytesExt};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::motions::Quaternion;

/// A position in one motion's key runs, reading the engine's key structs (`SkeletonMotions.hpp:31`) and reporting how
/// much is left rather than panicking past the end.
pub(crate) struct SkeletonMotionCursor<'a> {
  bytes: &'a [u8],
  offset: usize,
}

impl<'a> SkeletonMotionCursor<'a> {
  /// Bytes one quantised rotation key occupies, `CKeyQR`: four `i16` components.
  pub(crate) const ROTATION_KEY_SIZE: usize = 8;
  /// Bytes one 16 bit translation key occupies, `CKeyQT16`.
  pub(crate) const TRANSLATION_KEY_16_SIZE: usize = 6;
  /// Bytes one 8 bit translation key occupies, `CKeyQT8`.
  pub(crate) const TRANSLATION_KEY_8_SIZE: usize = 3;
  /// Bytes of the checksum leading a keyed stream, which the engine caches the keys by and nothing here needs.
  pub(crate) const CHECKSUM_SIZE: usize = 4;

  /// Divisor rotation components are quantised by, `KEY_Quant` (`SkeletonMotionDefs.hpp:11`).
  pub(crate) const KEY_QUANT: f32 = 32767.0;

  pub(crate) fn new(bytes: &'a [u8]) -> Self {
    Self { bytes, offset: 0 }
  }

  pub(crate) fn is_exhausted(&self) -> bool {
    self.offset == self.bytes.len()
  }

  pub(crate) fn remaining(&self) -> usize {
    self.bytes.len().saturating_sub(self.offset)
  }

  pub(crate) fn skip(&mut self, size: usize) -> XrfResult {
    self.take(size).map(|_| ())
  }

  pub(crate) fn read_u8(&mut self) -> XrfResult<u8> {
    Ok(self.take(1)?[0])
  }

  pub(crate) fn read_rotation<T: ByteOrder>(&mut self) -> XrfResult<Quaternion> {
    let mut bytes: &[u8] = self.take(Self::ROTATION_KEY_SIZE)?;

    Ok(Quaternion {
      x: f32::from(bytes.read_i16::<T>()?) / Self::KEY_QUANT,
      y: f32::from(bytes.read_i16::<T>()?) / Self::KEY_QUANT,
      z: f32::from(bytes.read_i16::<T>()?) / Self::KEY_QUANT,
      w: f32::from(bytes.read_i16::<T>()?) / Self::KEY_QUANT,
    })
  }

  pub(crate) fn read_translation_key_16<T: ByteOrder>(&mut self) -> XrfResult<[f32; 3]> {
    let mut bytes: &[u8] = self.take(Self::TRANSLATION_KEY_16_SIZE)?;

    Ok([
      f32::from(bytes.read_i16::<T>()?),
      f32::from(bytes.read_i16::<T>()?),
      f32::from(bytes.read_i16::<T>()?),
    ])
  }

  pub(crate) fn read_translation_key_8(&mut self) -> XrfResult<[f32; 3]> {
    let bytes: &[u8] = self.take(Self::TRANSLATION_KEY_8_SIZE)?;

    Ok([
      f32::from(bytes[0] as i8),
      f32::from(bytes[1] as i8),
      f32::from(bytes[2] as i8),
    ])
  }

  pub(crate) fn read_vector<T: ByteOrder>(&mut self) -> XrfResult<Vector3d> {
    let mut bytes: &[u8] = self.take(3 * size_of::<f32>())?;

    Ok(Vector3d {
      x: bytes.read_f32::<T>()?,
      y: bytes.read_f32::<T>()?,
      z: bytes.read_f32::<T>()?,
    })
  }

  fn take(&mut self, size: usize) -> XrfResult<&'a [u8]> {
    let end: usize = self.offset + size;

    if end > self.bytes.len() {
      return Err(XrfError::new_parsing_error(format!(
        "Motion keys end early: {size} more bytes wanted at offset {}, {} available",
        self.offset,
        self.remaining()
      )));
    }

    let taken: &[u8] = &self.bytes[self.offset..end];

    self.offset = end;

    Ok(taken)
  }
}
