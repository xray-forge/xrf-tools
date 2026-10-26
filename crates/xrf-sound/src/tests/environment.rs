use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::{EaxEnvironment, SoundEnvironment, SoundEnvironmentFile};

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
    bytes.extend_from_slice(&8u32.to_le_bytes());
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

fn read(environments: &[Vec<u8>]) -> XrfResult<SoundEnvironmentFile> {
  SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(library(environments)?)
}

fn write(file: &SoundEnvironmentFile) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  writer.flush_raw_into_buffer()
}

#[test]
fn reads_every_preset_of_a_library() -> XrfResult {
  let file: SoundEnvironmentFile = read(&[environment(4, "cave"), environment(4, "sewerpipe")])?;

  assert_eq!(file.environments.len(), 2);
  assert_eq!(file.environments[0].name, "cave");
  assert_eq!(file.environments[0].decay_time, 3.0);
  assert_eq!(file.environments[0].environment, Some(EaxEnvironment::Cave));
  assert_eq!(file.environments[0].get_version(), 4);

  Ok(())
}

#[test]
fn a_version_three_preset_names_no_eax_room_of_its_own() -> XrfResult {
  let file: SoundEnvironmentFile = read(&[environment(3, "room")])?;

  assert_eq!(file.environments[0].environment, None);
  assert_eq!(file.environments[0].get_version(), 3);

  Ok(())
}

#[test]
fn finds_a_preset_the_way_the_engine_matches_a_name() -> XrfResult {
  // `SoundEnvironment_LIB::GetID` compares with `xr_stricmp`, so a level naming a preset in another case still
  // reaches it.
  let file: SoundEnvironmentFile = read(&[environment(4, "room"), environment(4, "cave")])?;

  assert!(file.find_environment("CAVE").is_some());
  assert_eq!(file.get_environment_index("Cave"), Some(1));
  assert!(file.find_environment("hangar").is_none());

  Ok(())
}

#[test]
fn reads_presets_by_chunk_id_whatever_order_the_file_holds_them_in() -> XrfResult {
  // `SoundEnvironment_LIB::Load` opens chunk 0, then 1, by id, and a level addresses a preset by that number.
  let bytes: Vec<u8> = [chunk(1, &environment(4, "cave"))?, chunk(0, &environment(4, "room"))?].concat();
  let file: SoundEnvironmentFile = SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(file.get_environment_index("room"), Some(0));
  assert_eq!(file.get_environment_index("cave"), Some(1));

  Ok(())
}

#[test]
fn refuses_a_preset_chunk_the_engine_never_reaches() -> XrfResult {
  let gap: Vec<u8> = [chunk(0, &environment(4, "room"))?, chunk(2, &environment(4, "cave"))?].concat();
  let repeat: Vec<u8> = [chunk(0, &environment(4, "room"))?, chunk(0, &environment(4, "cave"))?].concat();

  assert_eq!(
    SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(gap)
      .unwrap_err()
      .to_string(),
    "Invalid error: Sound environment library has no chunk 1, so the engine never loads chunk 2"
  );
  assert_eq!(
    SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(repeat)
      .unwrap_err()
      .to_string(),
    "Invalid error: Sound environment library repeats chunk 0, which the engine never loads"
  );

  Ok(())
}

#[test]
fn refuses_a_preset_older_than_the_engine_reads() {
  // `CSoundRender_Environment::load` returns false below version 3 and the library drops the record, so a reader
  // that answered would list a preset the engine does not have.
  assert!(read(&[environment(2, "old")]).is_err());
}

#[test]
fn refuses_a_preset_newer_than_it_writes_back() {
  assert!(read(&[environment(5, "new")]).is_err());
}

#[test]
fn writes_a_library_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = library(&[environment(4, "cave"), environment(3, "room")])?;
  let file: SoundEnvironmentFile = SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

  assert_eq!(write(&file)?, original);

  Ok(())
}

#[test]
fn normalizes_a_library_to_the_version_the_engine_saves() -> XrfResult {
  let mut file: SoundEnvironmentFile = read(&[environment(3, "room")])?;

  file.normalize();

  assert_eq!(file.environments[0].environment, Some(EaxEnvironment::Generic));
  assert_eq!(
    SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(write(&file)?)?.environments[0].get_version(),
    4
  );

  Ok(())
}

#[test]
fn makes_presets_as_the_engine_does() {
  let default: SoundEnvironment = SoundEnvironment::new("default");
  let mut identity: SoundEnvironment = SoundEnvironment::new("identity");

  identity.set_identity();

  assert_eq!(default.room, -1000.0);
  assert_eq!(default.decay_time, 1.49);
  assert_eq!(default.environment, Some(EaxEnvironment::Generic));
  assert_eq!(identity.room, -10000.0);
  assert_eq!(
    identity.get_parameters()[1..],
    default.get_parameters()[1..],
    "identity silences the room and changes nothing else"
  );
}

#[test]
fn set_default_keeps_the_name() {
  let mut environment: SoundEnvironment = SoundEnvironment::new("cave");

  environment.reverb = 1500.0;
  environment.set_default();

  assert_eq!(environment, SoundEnvironment::new("cave"));
}

#[test]
fn clamps_to_the_ranges_eax_accepts() {
  let mut environment: SoundEnvironment = SoundEnvironment {
    room: 100.0,
    decay_time: 0.0,
    environment_size: 500.0,
    ..SoundEnvironment::default()
  };

  environment.clamp();

  assert_eq!(environment.room, 0.0);
  assert_eq!(environment.decay_time, 0.1);
  assert_eq!(environment.environment_size, 100.0);
}

#[test]
fn blends_towards_another_preset_keeping_its_own_name_and_room() {
  let from: SoundEnvironment = SoundEnvironment::new("plain");
  let to: SoundEnvironment = SoundEnvironment {
    name: String::from("cave"),
    decay_time: 3.49,
    environment: Some(EaxEnvironment::Cave),
    ..SoundEnvironment::default()
  };
  let blended: SoundEnvironment = from.lerp(&to, 0.5);

  assert_eq!(blended.name, "plain");
  assert_eq!(blended.environment, Some(EaxEnvironment::Generic));
  assert!((blended.decay_time - 2.49).abs() < 1e-6);
  assert_eq!(blended.room, from.room);
}

#[test]
fn edits_a_library_as_the_sdk_does() -> XrfResult {
  let mut file: SoundEnvironmentFile = SoundEnvironmentFile::default();

  file.append_environment("cave")?.decay_time = 3.0;
  file.clone_environment("cave", "mine")?;
  file.append_environment("room")?;

  assert_eq!(file.find_environment("mine").map(|it| it.decay_time), Some(3.0));
  assert!(
    file.append_environment("CAVE").is_err(),
    "the engine would never reach a second cave"
  );
  assert!(file.append_environment("").is_err());
  assert!(file.clone_environment("hangar", "hall").is_err());

  file.rename_environment("mine", "Mine")?;

  assert!(file.rename_environment("Mine", "room").is_err());

  file.find_environment_mut("ROOM").expect("the room was appended").reverb = 0.0;

  assert_eq!(file.remove_environment("cave")?.name, "cave");
  assert_eq!(
    file
      .environments
      .iter()
      .map(|environment| environment.name.as_str())
      .collect::<Vec<_>>(),
    ["Mine", "room"]
  );
  assert!(file.remove_environment("cave").is_err());

  Ok(())
}

#[test]
fn names_eax_rooms_as_the_sdk_does() {
  assert_eq!(EaxEnvironment::from(21), EaxEnvironment::SewerPipe);
  assert_eq!(EaxEnvironment::SewerPipe.label(), "Sewer Pipe");
  assert_eq!(EaxEnvironment::from(26), EaxEnvironment::Unknown(26));

  for environment in EaxEnvironment::NAMED {
    assert_eq!(EaxEnvironment::from(u32::from(environment)), environment);
  }
}
