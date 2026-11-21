use crate::bytes::xray_bytes::XRayBytes;

/// A value stored as exactly `SIZE` bytes, every one of whose byte patterns is a value: the one statement of its layout.
///
/// Arrays of records read and write in one pass over their bytes, where they lie
/// ([`crate::ChunkReader::read_records`], [`crate::ChunkWriter::write_records`]), rather than one stream call per field,
/// which is about nine times slower for a vertex-sized record. A record that also reads on its own implements
/// [`crate::ChunkReadWrite`] through [`Self::decode`] and [`Self::encode`], so the two cannot drift.
///
/// `SIZE` is inferred where the records' type is known, so a call names only the type:
/// `let faces: Vec<Face> = reader.read_records(count, "faces")?`.
pub trait ChunkRecord<const SIZE: usize>: Sized {
  fn decode(bytes: &[u8; SIZE]) -> Self;

  fn encode(&self) -> [u8; SIZE];
}

impl ChunkRecord<2> for u16 {
  #[inline]
  fn decode(bytes: &[u8; 2]) -> Self {
    XRayBytes::read_u16(bytes)
  }

  #[inline]
  fn encode(&self) -> [u8; 2] {
    self.to_le_bytes()
  }
}

impl ChunkRecord<4> for u32 {
  #[inline]
  fn decode(bytes: &[u8; 4]) -> Self {
    XRayBytes::read_u32(bytes)
  }

  #[inline]
  fn encode(&self) -> [u8; 4] {
    self.to_le_bytes()
  }
}

impl ChunkRecord<4> for f32 {
  #[inline]
  fn decode(bytes: &[u8; 4]) -> Self {
    XRayBytes::read_f32(bytes)
  }

  #[inline]
  fn encode(&self) -> [u8; 4] {
    self.to_le_bytes()
  }
}
