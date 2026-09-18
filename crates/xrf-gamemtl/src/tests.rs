use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::gamemtl_file::GameMtlFile;
use crate::gamemtl_material::GameMtlMaterial;
use crate::gamemtl_pair::GameMtlPair;

/// Frames a payload as one X-Ray chunk.
fn chunk(id: u32, payload: &[u8]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.buffer = payload.to_vec();

  writer.flush_chunk_into_buffer::<XRayByteOrder>(id)
}

/// A terminated Windows-1251 string.
fn string(value: &str) -> Vec<u8> {
  let mut bytes: Vec<u8> = value.as_bytes().to_vec();

  bytes.push(0);

  bytes
}

/// A run of little-endian floats.
fn floats(values: &[f32]) -> Vec<u8> {
  values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

/// One material, in the chunk order every shipped library writes.
fn material(id: u32, name: &str, description: Option<&str>, multiplayer: bool) -> XrfResult<Vec<u8>> {
  let mut main: Vec<u8> = id.to_le_bytes().to_vec();

  main.extend(string(name));

  let mut bytes: Vec<u8> = chunk(GameMtlMaterial::MAIN_CHUNK_ID, &main)?;

  if let Some(description) = description {
    bytes.extend(chunk(GameMtlMaterial::DESCRIPTION_CHUNK_ID, &string(description))?);
  }

  bytes.extend(chunk(GameMtlMaterial::FLAGS_CHUNK_ID, &0x2000_0009u32.to_le_bytes())?);
  bytes.extend(chunk(
    GameMtlMaterial::PHYSICS_CHUNK_ID,
    &floats(&[1.0, 1.0, 1.0, 0.0, 0.1]),
  )?);
  bytes.extend(chunk(
    GameMtlMaterial::FACTORS_CHUNK_ID,
    &floats(&[0.5, 1.0, 0.0, 0.25]),
  )?);

  if multiplayer {
    bytes.extend(chunk(GameMtlMaterial::FACTORS_MP_CHUNK_ID, &floats(&[0.75]))?);
  }

  bytes.extend(chunk(GameMtlMaterial::FLOTATION_CHUNK_ID, &floats(&[1.0]))?);
  bytes.extend(chunk(GameMtlMaterial::INJURIOUS_CHUNK_ID, &floats(&[0.0]))?);
  bytes.extend(chunk(GameMtlMaterial::DENSITY_CHUNK_ID, &floats(&[0.0]))?);

  Ok(bytes)
}

/// One pair, in the chunk order every shipped library writes.
fn pair(id: u32, parent: u32) -> XrfResult<Vec<u8>> {
  let mut head: Vec<u8> = Vec::new();

  head.extend_from_slice(&0u32.to_le_bytes());
  head.extend_from_slice(&1u32.to_le_bytes());
  head.extend_from_slice(&id.to_le_bytes());
  head.extend_from_slice(&parent.to_le_bytes());
  head.extend_from_slice(&0x46u32.to_le_bytes());

  let mut bytes: Vec<u8> = chunk(GameMtlPair::PAIR_CHUNK_ID, &head)?;

  bytes.extend(chunk(GameMtlPair::BREAKING_CHUNK_ID, &string("materials\\brk_1"))?);
  bytes.extend(chunk(GameMtlPair::STEP_CHUNK_ID, &string("materials\\step_1"))?);

  let mut collide: Vec<u8> = string("materials\\hit_1");

  collide.extend(string("effects\\sparks"));
  collide.extend(string("wm\\wm_hit"));

  bytes.extend(chunk(GameMtlPair::COLLIDE_CHUNK_ID, &collide)?);

  Ok(bytes)
}

/// A whole library holding the materials and pairs it is given.
fn library(materials: &[Vec<u8>], pairs: &[Vec<u8>]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(GameMtlFile::VERSION_CHUNK_ID, &1u16.to_le_bytes())?;
  let mut autoincrement: Vec<u8> = (materials.len() as u32).to_le_bytes().to_vec();

  autoincrement.extend_from_slice(&(pairs.len() as u32).to_le_bytes());

  bytes.extend(chunk(GameMtlFile::AUTOINCREMENT_CHUNK_ID, &autoincrement)?);

  let mut materials_chunk: Vec<u8> = Vec::new();

  for (index, material) in materials.iter().enumerate() {
    materials_chunk.extend(chunk(index as u32, material)?);
  }

  bytes.extend(chunk(GameMtlFile::MATERIALS_CHUNK_ID, &materials_chunk)?);

  let mut pairs_chunk: Vec<u8> = Vec::new();

  for (index, item) in pairs.iter().enumerate() {
    pairs_chunk.extend(chunk(index as u32, item)?);
  }

  bytes.extend(chunk(GameMtlFile::PAIRS_CHUNK_ID, &pairs_chunk)?);

  Ok(bytes)
}

/// A library holding one of each, which is the shape the round trips are taken over.
fn sample() -> XrfResult<Vec<u8>> {
  library(
    &[material(0, "default", Some("everything else"), true)?],
    &[pair(0, GameMtlPair::NO_PARENT_ID)?],
  )
}

#[test]
fn reads_a_material_and_its_optional_chunks() -> XrfResult {
  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(sample()?)?;

  assert_eq!(file.version, GameMtlFile::CURRENT_VERSION);
  assert_eq!(file.materials.len(), 1);
  assert_eq!(file.materials[0].name, "default");
  assert_eq!(file.materials[0].description.as_deref(), Some("everything else"));
  assert_eq!(file.materials[0].shoot_factor_mp, Some(0.75));
  assert_eq!(file.materials[0].acoustics, None);

  Ok(())
}

#[test]
fn names_the_flags_a_material_sets() -> XrfResult {
  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(sample()?)?;

  assert_eq!(
    file.materials[0].get_named_flags(),
    vec!["breakable", "skidmark", "shootable"]
  );

  Ok(())
}

#[test]
fn distinguishes_an_absent_multiplayer_factor_from_one_declared_as_zero() -> XrfResult {
  // `SGameMtl::Load` falls back to the single-player factor when the chunk is absent, so the two are not the same
  // reading and a writer that invented one would change what the engine does.
  let file: GameMtlFile =
    GameMtlFile::read_from_bytes::<XRayByteOrder>(library(&[material(0, "default", None, false)?], &[])?)?;

  assert_eq!(file.materials[0].shoot_factor_mp, None);
  assert_eq!(file.materials[0].description, None);

  Ok(())
}

#[test]
fn reads_a_pair_and_the_lists_it_names() -> XrfResult {
  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(sample()?)?;

  assert_eq!(file.pairs.len(), 1);
  assert_eq!(file.pairs[0].material_a, 0);
  assert_eq!(file.pairs[0].material_b, 1);
  assert_eq!(file.pairs[0].collide_particles, "effects\\sparks");
  assert!(!file.pairs[0].has_parent());
  assert_eq!(file.get_inheriting_pairs_count(), 0);

  Ok(())
}

#[test]
fn a_pair_naming_a_parent_is_counted_as_inheriting() -> XrfResult {
  let file: GameMtlFile =
    GameMtlFile::read_from_bytes::<XRayByteOrder>(library(&[material(0, "default", None, true)?], &[pair(1, 0)?])?)?;

  assert!(file.pairs[0].has_parent());
  assert_eq!(file.get_inheriting_pairs_count(), 1);

  Ok(())
}

#[test]
fn finds_a_material_by_the_number_a_collision_face_stores() -> XrfResult {
  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(library(
    &[material(7, "concrete", None, true)?, material(9, "metal", None, true)?],
    &[],
  )?)?;

  assert_eq!(file.find_material(9).map(|it| it.name.as_str()), Some("metal"));
  assert!(file.find_material(8).is_none());

  Ok(())
}

#[test]
fn writes_a_library_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = sample()?;
  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(original.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, original);

  Ok(())
}

#[test]
fn refuses_a_version_the_engine_gives_up_on() -> XrfResult {
  // `CGameMtlLibrary::Load` logs and loads nothing for any version but 1, so a reader that answered would describe a
  // library the game does not have.
  let mut bytes: Vec<u8> = chunk(GameMtlFile::VERSION_CHUNK_ID, &2u16.to_le_bytes())?;

  bytes.extend(chunk(GameMtlFile::AUTOINCREMENT_CHUNK_ID, &[0; 8])?);

  assert!(GameMtlFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}
