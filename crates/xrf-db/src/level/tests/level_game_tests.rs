use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::level::level_game_file::LevelGameFile;
use crate::level::level_game_way::LevelGameWay;
use crate::level::tests::fixtures::{chunk, floats, string};

/// One patrol path, in the chunk order the editor emits: version, name, points, links.
fn way_bytes(name: &str, points: &[&str]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(LevelGameWay::VERSION_CHUNK_ID, &0x0013u16.to_le_bytes())?;

  bytes.extend(chunk(LevelGameWay::NAME_CHUNK_ID, &string(name))?);

  let mut point_bytes: Vec<u8> = (points.len() as u16).to_le_bytes().to_vec();

  for point in points {
    point_bytes.extend(floats(&[1.0, 2.0, 3.0]));
    point_bytes.extend_from_slice(&0u32.to_le_bytes());
    point_bytes.extend(string(point));
  }

  bytes.extend(chunk(LevelGameWay::POINTS_CHUNK_ID, &point_bytes)?);
  bytes.extend(chunk(LevelGameWay::LINKS_CHUNK_ID, &0u16.to_le_bytes())?);

  Ok(bytes)
}

/// One respawn point, including the profile string every record stores.
fn rpoint_bytes(kind: u8, profile: &str) -> Vec<u8> {
  let mut bytes: Vec<u8> = floats(&[1.0, 2.0, 3.0, 0.0, 0.0, 0.0]);

  bytes.push(0);
  bytes.push(kind);
  bytes.extend_from_slice(&u16::MAX.to_le_bytes());
  bytes.extend(string(profile));

  bytes
}

/// A level's game data: the path table, then the respawn table, which is the order every shipped file uses.
fn file_bytes(ways: &[(&str, Vec<&str>)], rpoints: &[(u8, &str)]) -> XrfResult<Vec<u8>> {
  let mut way_table: Vec<u8> = Vec::new();

  for (index, (name, points)) in ways.iter().enumerate() {
    way_table.extend(chunk(index as u32, &way_bytes(name, points)?)?);
  }

  let mut rpoint_table: Vec<u8> = Vec::new();

  for (index, (kind, profile)) in rpoints.iter().enumerate() {
    rpoint_table.extend(chunk(index as u32, &rpoint_bytes(*kind, profile))?);
  }

  let mut bytes: Vec<u8> = chunk(LevelGameFile::WAYS_CHUNK_ID, &way_table)?;

  bytes.extend(chunk(LevelGameFile::RPOINTS_CHUNK_ID, &rpoint_table)?);

  Ok(bytes)
}

#[test]
fn level_game_data_reads_both_of_its_tables() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(
    &[("patrol_1", vec!["wp00", "wp01"])],
    &[(0, ""), (2, "1_medkit_10_sec")],
  )?;

  let file: LevelGameFile = LevelGameFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(file.ways.len(), 1);
  assert_eq!(file.ways[0].name, "patrol_1");
  assert_eq!(file.get_way_points_count(), 2);
  assert_eq!(file.rpoints.len(), 2);
  assert_eq!(file.rpoints[0].get_kind_label(), Some("actor spawn"));
  assert_eq!(file.rpoints[1].profile, "1_medkit_10_sec");
  assert_eq!(file.get_profiled_rpoints_count(), 1);

  Ok(())
}

#[test]
fn a_patrol_path_carries_no_type_chunk_because_no_shipped_one_does() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[("patrol_1", vec!["wp00"])], &[])?;
  let file: LevelGameFile = LevelGameFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(file.ways[0].kind, None);

  Ok(())
}

#[test]
fn every_respawn_point_stores_a_profile_even_where_the_engine_never_reads_one() -> XrfResult {
  // An actor spawn's is empty, and the engine skips it - but the byte is there, and a reader that followed the
  // engine would leave it behind and fail to write the file back.
  let bytes: Vec<u8> = file_bytes(&[], &[(0, "")])?;
  let file: LevelGameFile = LevelGameFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  assert_eq!(file.rpoints[0].profile, "");

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn level_game_data_is_written_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(
    &[("patrol_1", vec!["wp00", "wp01"])],
    &[(0, ""), (2, "1_medkit_10_sec")],
  )?;

  let file: LevelGameFile = LevelGameFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_level_carrying_one_table_and_not_the_other_still_reads() -> XrfResult {
  // Every shipped file holds both chunks and leaves one empty, which is the single-player shape.
  let file: LevelGameFile =
    LevelGameFile::read_from_bytes::<XRayByteOrder>(file_bytes(&[("patrol_1", vec!["wp00"])], &[])?)?;

  assert!(file.rpoints.is_empty());
  assert_eq!(file.ways.len(), 1);

  Ok(())
}
