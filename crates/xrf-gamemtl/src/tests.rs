use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::gamemtl_acoustics::GameMtlAcoustics;
use crate::gamemtl_file::GameMtlFile;
use crate::gamemtl_material::GameMtlMaterial;
use crate::gamemtl_pair::GameMtlPair;
use crate::gamemtl_version::GameMtlVersion;

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

/// One material, in the chunk order every shipped library writes, with the physics chunk it is given.
fn material_with_physics(id: u32, name: &str, description: Option<&str>, physics: &[f32]) -> XrfResult<Vec<u8>> {
  let mut main: Vec<u8> = id.to_le_bytes().to_vec();

  main.extend(string(name));

  let mut bytes: Vec<u8> = chunk(GameMtlMaterial::MAIN_CHUNK_ID, &main)?;

  if let Some(description) = description {
    bytes.extend(chunk(GameMtlMaterial::DESCRIPTION_CHUNK_ID, &string(description))?);
  }

  bytes.extend(chunk(GameMtlMaterial::FLAGS_CHUNK_ID, &0x2000_0009u32.to_le_bytes())?);
  bytes.extend(chunk(GameMtlMaterial::PHYSICS_CHUNK_ID, &floats(physics))?);
  bytes.extend(chunk(
    GameMtlMaterial::FACTORS_CHUNK_ID,
    &floats(&[0.5, 1.0, 0.0, 0.25]),
  )?);
  bytes.extend(chunk(GameMtlMaterial::FACTORS_MP_CHUNK_ID, &floats(&[0.75]))?);
  bytes.extend(chunk(GameMtlMaterial::FLOTATION_CHUNK_ID, &floats(&[1.0]))?);
  bytes.extend(chunk(GameMtlMaterial::INJURIOUS_CHUNK_ID, &floats(&[0.0]))?);
  bytes.extend(chunk(GameMtlMaterial::DENSITY_CHUNK_ID, &floats(&[0.0]))?);

  Ok(bytes)
}

/// One material, in the chunk order every shipped library writes.
fn material(id: u32, name: &str, description: Option<&str>) -> XrfResult<Vec<u8>> {
  material_with_physics(id, name, description, &[1.0, 1.0, 1.0, 0.0, 0.1])
}

/// One pair between the first two material numbers, in the chunk order every shipped library writes.
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
fn sample() -> XrfResult<GameMtlFile> {
  GameMtlFile::read_from_bytes::<XRayByteOrder>(library(
    &[material(0, "default", Some("everything else"))?],
    &[pair(0, GameMtlPair::NO_PARENT_ID)?],
  )?)
}

#[test]
fn reads_a_material_and_its_optional_chunks() -> XrfResult {
  let file: GameMtlFile = sample()?;

  assert_eq!(file.version, GameMtlFile::CURRENT_VERSION);
  assert_eq!(file.materials.len(), 1);
  assert_eq!(file.materials[0].name, "default");
  assert_eq!(file.materials[0].description.as_deref(), Some("everything else"));
  assert_eq!(file.materials[0].shoot_factor_mp, Some(0.75));
  assert_eq!(file.materials[0].acoustics, None);

  Ok(())
}

#[test]
fn reads_a_material_declaring_only_the_chunks_the_engine_asserts_on() -> XrfResult {
  let mut main: Vec<u8> = 3u32.to_le_bytes().to_vec();

  main.extend(string("materials\\grass"));

  let mut bytes: Vec<u8> = chunk(GameMtlMaterial::MAIN_CHUNK_ID, &main)?;

  bytes.extend(chunk(GameMtlMaterial::FLAGS_CHUNK_ID, &0u32.to_le_bytes())?);
  bytes.extend(chunk(GameMtlMaterial::PHYSICS_CHUNK_ID, &floats(&[0.0; 5]))?);
  bytes.extend(chunk(GameMtlMaterial::FACTORS_CHUNK_ID, &floats(&[0.0; 4]))?);

  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(library(&[bytes], &[])?)?;
  let material: &GameMtlMaterial = &file.materials[0];

  // Each absent chunk reads as absent rather than as the engine's fallback, which a writer would otherwise invent.
  assert_eq!(material.description, None);
  assert_eq!(material.flotation_factor, None);
  assert_eq!(material.injurious_speed, None);
  assert_eq!(material.density_factor, None);
  assert_eq!(material.shoot_factor_mp, None);
  assert_eq!(material.acoustics, None);
  assert_eq!(file.get_game_version(), GameMtlVersion::ShadowOfChernobyl);

  Ok(())
}

#[test]
fn names_every_flag_the_engine_names() -> XrfResult {
  let mut material: GameMtlMaterial = sample()?.materials.remove(0);

  assert_eq!(material.get_named_flags(), ["breakable", "skidmark", "shootable"]);

  // The editor sets the top two on every save, so a reader that left them out would drop them from shipped libraries.
  material.flags = GameMtlMaterial::FLAG_TRANSPARENT | GameMtlMaterial::FLAG_SLOW_DOWN | (1 << 6);

  assert_eq!(material.get_named_flags(), ["transparent", "slow down"]);

  Ok(())
}

#[test]
fn falls_back_to_what_the_engine_plays_with_for_an_absent_chunk() -> XrfResult {
  let mut material: GameMtlMaterial = sample()?.materials.remove(0);

  material.flotation_factor = None;
  material.injurious_speed = None;
  material.density_factor = None;
  material.shoot_factor_mp = None;

  assert_eq!(material.get_flotation_factor(), 1.0);
  assert_eq!(material.get_injurious_speed(), 0.0);
  assert_eq!(material.get_density_factor(), 0.0);
  assert_eq!(material.get_shoot_factor_mp(), material.shoot_factor);

  Ok(())
}

#[test]
fn gives_a_material_without_acoustics_the_engine_default_for_its_name() -> XrfResult {
  let mut material: GameMtlMaterial = sample()?.materials.remove(0);

  material.name = "materials\\water".into();

  assert_eq!(
    material.get_acoustics(),
    GameMtlAcoustics::new([0.01, 0.02, 0.03], 0.05, [0.99, 0.98, 0.97])
  );

  // A name the table does not hold takes the first entry, absorbing as much as the material occludes.
  material.name = "materials\\Water".into();

  assert_eq!(
    material.get_acoustics(),
    GameMtlAcoustics::new([0.25; 3], 0.05, [0.100, 0.050, 0.030])
  );

  Ok(())
}

#[test]
fn tells_the_game_apart_by_the_newest_chunk_a_material_carries() -> XrfResult {
  let mut file: GameMtlFile = sample()?;

  assert_eq!(file.get_game_version(), GameMtlVersion::CallOfPripyat);

  file.materials[0].shoot_factor_mp = None;

  assert_eq!(file.get_game_version(), GameMtlVersion::ClearSky);

  file.materials[0].density_factor = None;

  assert_eq!(file.get_game_version(), GameMtlVersion::ShadowOfChernobyl);

  file.materials.clear();

  assert_eq!(file.get_game_version(), GameMtlVersion::ShadowOfChernobyl);

  Ok(())
}

#[test]
fn distinguishes_an_absent_description_from_an_empty_one() -> XrfResult {
  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(library(
    &[material(0, "default", None)?, material(1, "metal", Some(""))?],
    &[],
  )?)?;

  assert_eq!(file.materials[0].description, None);
  assert_eq!(file.materials[1].description.as_deref(), Some(""));

  Ok(())
}

#[test]
fn reads_a_pair_and_the_lists_it_names() -> XrfResult {
  let file: GameMtlFile = sample()?;

  assert_eq!(file.pairs.len(), 1);
  assert_eq!(file.pairs[0].material_a, 0);
  assert_eq!(file.pairs[0].material_b, 1);
  assert_eq!(file.pairs[0].collide_particles, "effects\\sparks");
  assert_eq!(
    file.pairs[0].get_own_properties(),
    ["breaking sounds", "step sounds", "collide marks"]
  );
  assert!(!file.pairs[0].has_parent());
  assert_eq!(file.get_inheriting_pairs_count(), 0);

  Ok(())
}

#[test]
fn a_pair_naming_a_parent_is_counted_as_inheriting() -> XrfResult {
  let file: GameMtlFile =
    GameMtlFile::read_from_bytes::<XRayByteOrder>(library(&[material(0, "default", None)?], &[pair(1, 0)?])?)?;

  assert!(file.pairs[0].has_parent());
  assert_eq!(file.get_inheriting_pairs_count(), 1);

  Ok(())
}

#[test]
fn finds_a_pair_whichever_way_round_its_materials_are_named() -> XrfResult {
  let file: GameMtlFile = sample()?;

  assert_eq!(file.find_pair(1, 0).map(|it| it.id), Some(0));
  assert_eq!(file.find_pair(0, 1).map(|it| it.id), Some(0));
  assert!(file.find_pair(0, 0).is_none());

  Ok(())
}

#[test]
fn splits_a_list_as_the_engine_counts_and_trims_it() {
  assert_eq!(GameMtlPair::split_list(""), Vec::<&str>::new());
  assert_eq!(GameMtlPair::split_list("a, b ,c"), ["a", "b", "c"]);
  assert_eq!(GameMtlPair::split_list("a,"), ["a"]);
  // `_GetItemCount` stops at two commas in a row, so the engine loads an empty entry and never reaches `b`.
  assert_eq!(GameMtlPair::split_list("a,,b,c"), ["a", ""]);
}

#[test]
fn builds_a_library_numbering_on_from_its_highest_numbers() -> XrfResult {
  let file: GameMtlFile = GameMtlFile::new(
    vec![GameMtlMaterial::new(4, "default"), GameMtlMaterial::new(9, "metal")],
    vec![GameMtlPair::new(2, 4, 9)],
  );
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  let restored: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(writer.flush_raw_into_buffer()?)?;

  assert_eq!(restored, file);
  assert_eq!((restored.material_index, restored.material_pair_index), (10, 3));
  assert_eq!(restored.get_game_version(), GameMtlVersion::CallOfPripyat);
  assert_eq!(restored.pairs[0].get_own_properties().len(), 5);
  assert_eq!(GameMtlFile::new(Vec::new(), Vec::new()).material_index, 0);

  Ok(())
}

#[test]
fn labels_a_pair_by_its_materials_and_counts_the_pairs_naming_each_list_entry() {
  let mut first: GameMtlPair = GameMtlPair::new(0, 0, 7);
  let mut second: GameMtlPair = GameMtlPair::new(1, 0, 0);

  first.collide_marks = String::from("wm\\a, wm\\b");
  second.collide_marks = String::from("wm\\a,,wm\\c");

  let file: GameMtlFile = GameMtlFile::new(vec![GameMtlMaterial::new(0, "default")], vec![first, second]);

  assert_eq!(file.get_pair_label(&file.pairs[0]), "default - 7");
  assert_eq!(
    file
      .get_list_entries(|pair| &pair.collide_marks)
      .into_iter()
      .collect::<Vec<(&str, usize)>>(),
    [("wm\\a", 2), ("wm\\b", 1)]
  );
}

#[test]
fn finds_a_material_by_the_number_a_collision_face_stores() -> XrfResult {
  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(library(
    &[material(7, "concrete", None)?, material(9, "metal", None)?],
    &[],
  )?)?;

  assert_eq!(file.find_material(9).map(|it| it.name.as_str()), Some("metal"));
  assert!(file.find_material(8).is_none());

  Ok(())
}

#[test]
fn finds_a_material_by_name_regardless_of_ascii_case() -> XrfResult {
  let file: GameMtlFile = sample()?;

  assert_eq!(file.find_material_by_name("DEFAULT").map(|it| it.id), Some(0));
  assert!(file.find_material_by_name("default_object").is_none());

  Ok(())
}

#[test]
fn writes_a_library_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = library(
    &[material(0, "default", Some("everything else"))?],
    &[pair(0, GameMtlPair::NO_PARENT_ID)?],
  )?;
  let file: GameMtlFile = GameMtlFile::read_from_bytes::<XRayByteOrder>(original.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, original);

  Ok(())
}

#[test]
fn refuses_a_chunk_holding_more_than_the_engine_reads() -> XrfResult {
  // The engine reads five physics floats and walks on, but the writer could not give the sixth back.
  let bytes: Vec<u8> = library(
    &[material_with_physics(
      0,
      "default",
      None,
      &[1.0, 1.0, 1.0, 0.0, 0.1, 9.0],
    )?],
    &[],
  )?;

  assert!(GameMtlFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

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
