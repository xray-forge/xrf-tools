use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::sound_environment::SoundEnvironment;
use crate::sound_environment_file::SoundEnvironmentFile;

/// Frames a payload as one X-Ray chunk.
fn chunk(id: u32, payload: &[u8]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.buffer = payload.to_vec();

  writer.flush_chunk_into_buffer::<XRayByteOrder>(id)
}

/// One preset, as the library stores it.
fn environment(version: u32, name: &str) -> Vec<u8> {
  let mut bytes: Vec<u8> = version.to_le_bytes().to_vec();

  bytes.extend_from_slice(name.as_bytes());
  bytes.push(0);

  for index in 0..12 {
    bytes.extend_from_slice(&(index as f32).to_le_bytes());
  }

  if version >= SoundEnvironment::ENVIRONMENT_VERSION {
    bytes.extend_from_slice(&26u32.to_le_bytes());
  }

  bytes
}

/// A whole library holding the presets it is given.
fn library(environments: &[Vec<u8>]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = Vec::new();

  for (index, environment) in environments.iter().enumerate() {
    bytes.extend(chunk(index as u32, environment)?);
  }

  Ok(bytes)
}

#[test]
fn reads_every_preset_of_a_library() -> XrfResult {
  let file: SoundEnvironmentFile = SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(library(&[
    environment(4, "cave"),
    environment(4, "sewerpipe"),
  ])?)?;

  assert_eq!(file.environments.len(), 2);
  assert_eq!(file.environments[0].name, "cave");
  assert_eq!(file.environments[0].decay_time, 3.0);
  assert_eq!(file.environments[0].environment, Some(26));

  Ok(())
}

#[test]
fn a_version_three_preset_names_no_eax_preset_of_its_own() -> XrfResult {
  let file: SoundEnvironmentFile =
    SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(library(&[environment(3, "room")])?)?;

  assert_eq!(file.environments[0].environment, None);

  Ok(())
}

#[test]
fn finds_a_preset_the_way_the_engine_matches_a_name() -> XrfResult {
  // `SoundEnvironment_LIB::GetID` compares with `xr_stricmp`, so a level naming a preset in another case still
  // reaches it.
  let file: SoundEnvironmentFile =
    SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(library(&[environment(4, "cave")])?)?;

  assert!(file.find_environment("CAVE").is_some());
  assert!(file.find_environment("hangar").is_none());

  Ok(())
}

#[test]
fn refuses_a_preset_older_than_the_engine_reads() -> XrfResult {
  // `CSoundRender_Environment::load` returns false below version 3 and the library drops the record, so a reader
  // that answered would list a preset the engine does not have.
  assert!(SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(library(&[environment(2, "old")])?).is_err());

  Ok(())
}

#[test]
fn writes_a_library_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = library(&[environment(4, "cave"), environment(3, "room")])?;
  let file: SoundEnvironmentFile = SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(original.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, original);

  Ok(())
}
