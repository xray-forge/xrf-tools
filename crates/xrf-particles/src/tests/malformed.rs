//! Malformed libraries answer with an error rather than a panic, which a release build turns into an abort.
//!
//! Inputs are framed by hand, independent of the writers, so a reader cannot pass by agreeing with its own writer.

use xrf_chunk::{ChunkReadWrite, ChunkReader, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};

use crate::{ParticleEffect, ParticleGroup, ParticlesFile};

/// One chunk: little-endian id and size, then the payload.
fn chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  [&id.to_le_bytes()[..], &(payload.len() as u32).to_le_bytes(), payload].concat()
}

/// Every chunk an effect requires, and none of the optional ones.
fn effect_required_chunks() -> Vec<Vec<u8>> {
  vec![
    chunk(ParticleEffect::VERSION_CHUNK_ID, &1u16.to_le_bytes()),
    chunk(ParticleEffect::NAME_CHUNK_ID, b"fx\0"),
    chunk(ParticleEffect::MAX_PARTICLES_CHUNK_ID, &16u32.to_le_bytes()),
    chunk(ParticleEffect::ACTION_LIST_CHUNK_ID, &0u32.to_le_bytes()),
    chunk(ParticleEffect::FLAGS_CHUNK_ID, &0u32.to_le_bytes()),
    chunk(ParticleEffect::SPRITE_CHUNK_ID, b"shader\0texture\0"),
  ]
}

fn read_effect(chunks: &[Vec<u8>]) -> XrfResult<ParticleEffect> {
  ParticleEffect::read::<XRayByteOrder, _>(&mut ChunkReader::from_vec(chunks.concat())?)
}

/// Every chunk a group requires, and none of the optional ones.
fn group_required_chunks() -> Vec<Vec<u8>> {
  vec![
    chunk(ParticleGroup::VERSION_CHUNK_ID, &3u16.to_le_bytes()),
    chunk(ParticleGroup::NAME_CHUNK_ID, b"group\0"),
    chunk(ParticleGroup::FLAGS_CHUNK_ID, &0u32.to_le_bytes()),
    chunk(ParticleGroup::EFFECTS_CHUNK_ID, &0u32.to_le_bytes()),
    chunk(ParticleGroup::TIME_LIMIT_CHUNK_ID, &0f32.to_le_bytes()),
  ]
}

fn read_group(chunks: &[Vec<u8>]) -> XrfResult<ParticleGroup> {
  ParticleGroup::read::<XRayByteOrder, _>(&mut ChunkReader::from_vec(chunks.concat())?)
}

fn assert_parsing_error_naming<R: std::fmt::Debug>(result: XrfResult<R>, name: &str) {
  match result {
    Err(XrfError::Parsing { message }) => assert!(message.contains(name), "'{message}' should name {name}"),
    other => panic!("expected a parsing error naming {name}, got {other:?}"),
  }
}

#[test]
fn reads_an_effect_without_its_optional_chunks() -> XrfResult {
  let effect: ParticleEffect = read_effect(&effect_required_chunks())?;

  assert_eq!(effect.name, "fx");
  assert_eq!(effect.frame, None);
  assert_eq!(effect.description, None);

  Ok(())
}

#[test]
fn refuses_an_effect_missing_a_required_chunk() {
  let mut chunks: Vec<Vec<u8>> = effect_required_chunks();

  chunks.remove(0);

  assert_parsing_error_naming(read_effect(&chunks), "version");
}

/// Editor data is absent here: it is an opaque byte blob, so no payload of it is malformed.
#[test]
fn refuses_an_effect_whose_optional_chunks_are_malformed() {
  let malformed: [(u32, &str, Vec<u8>); 6] = [
    (ParticleEffect::FRAME_CHUNK_ID, "frame", vec![0; 3]),
    (ParticleEffect::TIME_LIMIT_CHUNK_ID, "time_limit", vec![0; 2]),
    (ParticleEffect::COLLISION_CHUNK_ID, "collision", vec![0; 1]),
    (ParticleEffect::VELOCITY_SCALE_CHUNK_ID, "velocity_scale", vec![0; 5]),
    (ParticleEffect::DESCRIPTION_CHUNK_ID, "description", b"creator".to_vec()),
    (ParticleEffect::ROTATION_CHUNK_ID, "rotation", vec![0; 5]),
  ];

  for (id, name, payload) in malformed {
    let mut chunks: Vec<Vec<u8>> = effect_required_chunks();

    chunks.push(chunk(id, &payload));

    assert_parsing_error_naming(read_effect(&chunks), name);
  }
}

#[test]
fn refuses_a_frame_chunk_with_bytes_left_over() {
  let mut chunks: Vec<Vec<u8>> = effect_required_chunks();

  // A whole frame is 28 bytes; one more is data the reader would otherwise have asserted on.
  chunks.push(chunk(ParticleEffect::FRAME_CHUNK_ID, &[0; 29]));

  assert_parsing_error_naming(read_effect(&chunks), "frame");
}

#[test]
fn reads_a_group_without_its_optional_chunks() -> XrfResult {
  let group: ParticleGroup = read_group(&group_required_chunks())?;

  assert_eq!(group.name, "group");
  assert_eq!(group.description, None);
  assert_eq!(group.effects_old, None);

  Ok(())
}

#[test]
fn refuses_a_group_whose_optional_chunks_are_malformed() {
  for (id, name) in [
    (ParticleGroup::DESCRIPTION_CHUNK_ID, "description"),
    (ParticleGroup::EFFECTS2_CHUNK_ID, "old effects"),
  ] {
    let mut chunks: Vec<Vec<u8>> = group_required_chunks();

    chunks.push(chunk(id, &[1, 0, 0, 0]));

    assert_parsing_error_naming(read_group(&chunks), name);
  }
}

/// The audit's probe: a correctly framed library whose one effect carries no chunks at all.
#[test]
fn refuses_a_library_holding_an_empty_effect() {
  let library: Vec<u8> = [chunk(1, &1u16.to_le_bytes()), chunk(3, &chunk(0, &[])), chunk(4, &[])].concat();

  assert_parsing_error_naming(ParticlesFile::read_from_bytes::<XRayByteOrder>(library), "version");
}

#[test]
fn refuses_a_library_header_with_bytes_left_over() {
  let library: Vec<u8> = [chunk(1, &[1, 0, 0]), chunk(3, &[]), chunk(4, &[])].concat();

  assert!(matches!(
    ParticlesFile::read_from_bytes::<XRayByteOrder>(library),
    Err(XrfError::ChunkNotEnded { .. })
  ));
}
