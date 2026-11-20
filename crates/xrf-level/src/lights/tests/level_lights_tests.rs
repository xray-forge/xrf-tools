use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::lights::level_light::LevelLight;
use crate::lights::level_light_role::LevelLightRole;
use crate::lights::level_lights_chunk::LevelLightsChunk;
use crate::lights::level_lights_file::LevelLightsFile;
use crate::tests::fixtures::{chunk, floats};

/// One compiled light, laid out exactly as the compiler blits `R_Light`.
fn light_bytes(kind: u16, range: f32) -> Vec<u8> {
  let mut bytes: Vec<u8> = kind.to_le_bytes().to_vec();

  bytes.extend_from_slice(&0u16.to_le_bytes());
  bytes.extend(floats(&[1.0, 0.9, 0.8]));
  bytes.extend(floats(&[10.0, 2.0, -5.0]));
  bytes.extend(floats(&[0.0, -1.0, 0.0]));
  bytes.extend(floats(&[range, range * range, 0.5, 1.0, 0.0, 0.0, 100.0]));
  bytes.extend(floats(&[0.0; 9]));

  bytes
}

/// A light chunk holding the given kinds.
fn lights_chunk(id: u32, kinds: &[u16]) -> XrfResult<Vec<u8>> {
  let mut payload: Vec<u8> = Vec::new();

  for kind in kinds {
    payload.extend(light_bytes(*kind, 8.0));
  }

  chunk(id, &payload)
}

#[test]
fn a_light_is_the_width_the_compiler_blits_it_at() {
  assert_eq!(LevelLight::SERIALIZED_SIZE, 104);
  assert_eq!(light_bytes(1, 8.0).len() as u64, LevelLight::SERIALIZED_SIZE);
}

#[test]
fn a_light_list_reads_every_chunk_as_the_run_of_lights_it_is() -> XrfResult {
  let mut bytes: Vec<u8> = lights_chunk(0, &[3, 1])?;

  bytes.extend(lights_chunk(LevelLightsChunk::HEMI_CHUNK_ID, &[1, 1, 1])?);

  let file: LevelLightsFile = LevelLightsFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(file.chunks.len(), 2);
  assert_eq!(file.get_lights_count(), 5);
  assert_eq!(file.chunks[0].get_id(), 0);
  assert_eq!(file.chunks[0].get_lights()[0].range, 8.0);

  Ok(())
}

#[test]
fn only_the_point_lights_of_the_header_chunk_reach_the_runtime() -> XrfResult {
  // `CLight_DB::LoadHemi` opens `fsL_HEADER` alone and keeps only `D3DLIGHT_POINT`.
  let mut bytes: Vec<u8> = lights_chunk(0, &[1, 1])?;

  bytes.extend(lights_chunk(LevelLightsChunk::HEMI_CHUNK_ID, &[1, 3, 1])?);

  let file: LevelLightsFile = LevelLightsFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(file.get_lights_count(), 5);
  assert_eq!(file.get_hemi_lights().count(), 2);

  Ok(())
}

#[test]
fn reads_each_role_s_lights_from_its_own_chunk_point_and_directional_alike() -> XrfResult {
  let mut bytes: Vec<u8> = lights_chunk(LevelLightRole::Rgb.get_chunk_id(), &[1])?;

  bytes.extend(lights_chunk(LevelLightRole::Hemi.get_chunk_id(), &[0, 0, 1])?);
  bytes.extend(lights_chunk(LevelLightRole::Sun.get_chunk_id(), &[0])?);

  let file: LevelLightsFile = LevelLightsFile::read_from_bytes::<XRayByteOrder>(bytes)?;
  let kinds = |role: LevelLightRole| -> Vec<u16> { file.get_role_lights(role).map(|light| light.kind).collect() };

  assert_eq!(kinds(LevelLightRole::Rgb), [LevelLight::KIND_POINT]);
  assert_eq!(kinds(LevelLightRole::Hemi), [0, 0, 1]);
  assert_eq!(kinds(LevelLightRole::Sun), [LevelLight::KIND_DIRECTIONAL]);
  assert!(
    file
      .get_role_lights(LevelLightRole::Sun)
      .all(LevelLight::is_directional)
  );

  Ok(())
}

#[test]
fn a_chunk_that_is_not_whole_lights_is_kept_verbatim_rather_than_refused() -> XrfResult {
  // `gamedata-cop-ee\levels\pripyat\build.lights` carries a two-byte chunk 10 ahead of the usual three, and the
  // engine never notices because it opens its one chunk by id.
  let mut bytes: Vec<u8> = chunk(10, &[7, 9])?;

  bytes.extend(lights_chunk(LevelLightsChunk::HEMI_CHUNK_ID, &[1])?);

  let file: LevelLightsFile = LevelLightsFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  assert_eq!(file.chunks.len(), 2);
  assert_eq!(
    file.chunks[0],
    LevelLightsChunk::Opaque {
      id: 10,
      data: vec![7, 9]
    }
  );
  assert!(file.chunks[0].get_lights().is_empty());
  assert_eq!(file.get_lights_count(), 1);

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes, "an unread chunk must survive a rewrite");

  Ok(())
}

#[test]
fn a_light_list_is_written_back_byte_for_byte() -> XrfResult {
  let mut bytes: Vec<u8> = lights_chunk(0, &[3, 1])?;

  bytes.extend(lights_chunk(LevelLightsChunk::HEMI_CHUNK_ID, &[1, 1])?);
  bytes.extend(lights_chunk(2, &[1])?);

  let file: LevelLightsFile = LevelLightsFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}
