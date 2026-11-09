use xrf_error::XrfResult;
use xrf_utils::encode_string_to_w1251_bytes;

use crate::field::chunk_field_kind::ChunkFieldKind;

/// One value as a [`crate::ChunkFieldWriter`] writes it, with the primitive that wrote it: what the engine's
/// `NET_Packet` saves as one field when an ini stream is attached to it.
#[derive(Clone, Debug, PartialEq)]
pub enum ChunkField {
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  F32(f32),
  Vector3([f32; 3]),
  String(String),
}

impl ChunkField {
  pub const fn get_kind(&self) -> ChunkFieldKind {
    match self {
      Self::U8(_) => ChunkFieldKind::U8,
      Self::U16(_) => ChunkFieldKind::U16,
      Self::U32(_) => ChunkFieldKind::U32,
      Self::U64(_) => ChunkFieldKind::U64,
      Self::F32(_) => ChunkFieldKind::F32,
      Self::Vector3(_) => ChunkFieldKind::Vector3,
      Self::String(_) => ChunkFieldKind::String,
    }
  }

  /// The bytes the value takes written as bytes: a string its Windows-1251 encoding and terminator.
  ///
  /// # Errors
  ///
  /// Returns an error for a string with a character Windows-1251 has no byte for.
  pub fn measure_bytes(&self) -> XrfResult<usize> {
    Ok(match self {
      Self::U8(_) => size_of::<u8>(),
      Self::U16(_) => size_of::<u16>(),
      Self::U32(_) => size_of::<u32>(),
      Self::U64(_) => size_of::<u64>(),
      Self::F32(_) => size_of::<f32>(),
      Self::Vector3(_) => size_of::<[f32; 3]>(),
      Self::String(value) => encode_string_to_w1251_bytes(value)?.len() + 1,
    })
  }
}
