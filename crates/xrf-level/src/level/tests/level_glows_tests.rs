use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::level::glows::level_glow::LevelGlow;
use crate::level::glows::level_glows_chunk::LevelGlowsChunk;
use crate::level::level_file::LevelFile;
use crate::level::level_header_chunk::LevelHeaderChunk;
use crate::tests::fixtures::{chunk, floats};

/// One glow record: a position, a radius and a shader row.
fn glow(position: [f32; 3], radius: f32, shader: u16) -> Vec<u8> {
  let mut bytes: Vec<u8> = floats(&[position[0], position[1], position[2], radius]);

  bytes.extend_from_slice(&shader.to_le_bytes());

  bytes
}

/// A whole `level` carrying a header and the glows.
fn level(glows: &[u8]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(LevelHeaderChunk::CHUNK_ID, &[14, 0, 1, 0])?;

  bytes.extend(chunk(LevelGlowsChunk::CHUNK_ID, glows)?);

  Ok(bytes)
}

#[test]
fn reads_every_glow_the_chunk_size_counts() -> XrfResult {
  let mut glows: Vec<u8> = glow([1.0, 2.0, 3.0], 0.5, 12);

  glows.extend(glow([-4.0, 0.0, 8.0], 1.25, 13));

  let read: LevelFile = LevelFile::read_from_bytes::<XRayByteOrder>(level(&glows)?)?;

  assert_eq!(
    read.get_glows(),
    [
      LevelGlow {
        position: Vector3d::new(1.0, 2.0, 3.0),
        radius: 0.5,
        shader: 12,
      },
      LevelGlow {
        position: Vector3d::new(-4.0, 0.0, 8.0),
        radius: 1.25,
        shader: 13,
      },
    ]
  );

  Ok(())
}

#[test]
fn a_level_without_the_chunk_has_no_glows() -> XrfResult {
  let read: LevelFile =
    LevelFile::read_from_bytes::<XRayByteOrder>(chunk(LevelHeaderChunk::CHUNK_ID, &[14, 0, 1, 0])?)?;

  assert_eq!(read.glows, None);
  assert!(read.get_glows().is_empty());

  Ok(())
}

#[test]
fn a_partial_glow_record_is_refused() -> XrfResult {
  let mut glows: Vec<u8> = glow([1.0, 2.0, 3.0], 0.5, 12);

  glows.push(0);

  let error: String = LevelFile::read_from_bytes::<XRayByteOrder>(level(&glows)?)
    .expect_err("a trailing byte is not a glow")
    .to_string();

  assert_eq!(
    error,
    "Invalid error: Unexpected level glows chunk of 19 bytes, which does not divide into 18 byte records"
  );

  Ok(())
}
