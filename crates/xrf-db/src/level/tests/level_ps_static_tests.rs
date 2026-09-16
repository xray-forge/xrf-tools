use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::level::level_ps_static_file::LevelPsStaticFile;
use crate::level::level_ps_static_placement::PS_STATIC_ALL_GAME_TYPES;
use crate::level::tests::fixtures::{chunk, string};

/// One static particle placement, at the width its version gives it.
fn placement_bytes(game_types: Option<u16>, effect: &str) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  if let Some(types) = game_types {
    bytes.extend_from_slice(&types.to_le_bytes());
  }

  bytes.extend(string(effect));

  for index in 0..16 {
    bytes.extend_from_slice(&(index as f32).to_le_bytes());
  }

  bytes
}

/// A placement list: a version chunk, then one numbered chunk per record.
fn file_bytes(placements: &[(Option<u16>, &str)]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(0, &1u32.to_le_bytes())?;

  for (index, (game_types, effect)) in placements.iter().enumerate() {
    bytes.extend(chunk(index as u32 + 1, &placement_bytes(*game_types, effect))?);
  }

  Ok(bytes)
}

#[test]
fn a_static_particle_reads_its_effect_and_the_place_it_sits() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[
    (Some(PS_STATIC_ALL_GAME_TYPES), "zones\\zone_acidic_idle"),
    (Some(24), "weapons\\generic_shoot"),
  ])?;

  let file: LevelPsStaticFile = LevelPsStaticFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(file.version, LevelPsStaticFile::CURRENT_VERSION);
  assert_eq!(file.placements.len(), 2);
  assert_eq!(file.placements[0].effect, "zones\\zone_acidic_idle");
  // The translation is the fourth row, which is elements twelve through fourteen.
  assert_eq!(file.placements[0].get_position().x, 12.0);

  Ok(())
}

#[test]
fn a_placement_restricted_to_some_modes_is_told_from_one_that_plays_in_all() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[
    (Some(PS_STATIC_ALL_GAME_TYPES), "zones\\zone_acidic_idle"),
    (Some(24), "weapons\\generic_shoot"),
  ])?;

  let file: LevelPsStaticFile = LevelPsStaticFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert!(file.placements[0].is_every_game_type());
  assert!(!file.placements[1].is_every_game_type());
  assert_eq!(file.get_restricted_count(), 1);

  Ok(())
}

#[test]
fn a_placement_list_is_written_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[(Some(PS_STATIC_ALL_GAME_TYPES), "zones\\zone_acidic_idle")])?;
  let file: LevelPsStaticFile = LevelPsStaticFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_placement_disagreeing_with_its_version_is_refused_rather_than_written() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[(Some(0), "zones\\zone_acidic_idle")])?;
  let mut file: LevelPsStaticFile = LevelPsStaticFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  file.placements[0].game_types = None;

  assert!(file.write::<XRayByteOrder>(&mut ChunkWriter::new()).is_err());

  Ok(())
}
