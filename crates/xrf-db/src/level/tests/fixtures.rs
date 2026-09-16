use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

/// Frames a payload as one X-Ray chunk.
pub(crate) fn chunk(id: u32, payload: &[u8]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.buffer = payload.to_vec();

  writer.flush_chunk_into_buffer::<XRayByteOrder>(id)
}

/// A run of little-endian floats.
pub(crate) fn floats(values: &[f32]) -> Vec<u8> {
  values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

/// A terminated Windows-1251 string, which every name in these formats is.
pub(crate) fn string(value: &str) -> Vec<u8> {
  let mut bytes: Vec<u8> = value.as_bytes().to_vec();

  bytes.push(0);

  bytes
}

/// A triangle of either occlusion mesh: three positions, then whatever trails them.
pub(crate) fn triangle(tail: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);

  bytes.extend_from_slice(tail);

  bytes
}
