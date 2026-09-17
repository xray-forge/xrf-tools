use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::spawn::level_spawn_file::LevelSpawnFile;

/// Frames a payload as one X-Ray chunk.
fn new_chunk(id: u32, payload: &[u8]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.buffer = payload.to_vec();

  writer.flush_chunk_into_buffer::<XRayByteOrder>(id)
}

/// One spawn packet, as `CSE_Abstract::Spawn_Write` frames it.
fn new_object(action: u16, section: &str, name: &str, position: [f32; 3], tail: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = action.to_le_bytes().to_vec();

  bytes.extend_from_slice(section.as_bytes());
  bytes.push(0);
  bytes.extend_from_slice(name.as_bytes());
  bytes.push(0);
  bytes.extend_from_slice(&[0, 0xFF]);

  for value in position {
    bytes.extend_from_slice(&value.to_le_bytes());
  }

  for _ in 0..3 {
    bytes.extend_from_slice(&0.0f32.to_le_bytes());
  }

  bytes.extend_from_slice(tail);

  bytes
}

/// A whole list out of the objects it is given, numbered as every shipped file numbers them.
fn new_list(objects: &[Vec<u8>]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = Vec::new();

  for (index, object) in objects.iter().enumerate() {
    bytes.extend(new_chunk(index as u32, object)?);
  }

  Ok(bytes)
}

/// A list holding three objects of two sections.
fn new_sample() -> XrfResult<Vec<u8>> {
  new_list(&[
    new_object(1, "graph_point", "fake_graph_point", [0.0, 0.0, 0.0], &[7; 12]),
    new_object(1, "m_stalker", "esc_wolf", [10.0, 1.0, -4.0], &[9; 40]),
    new_object(1, "graph_point", "fake_exit_point", [1.0, 0.0, 2.0], &[7; 12]),
  ])
}

#[test]
fn reads_every_object_of_a_flat_packet_run() -> XrfResult {
  let file: LevelSpawnFile = LevelSpawnFile::read_from_bytes::<XRayByteOrder>(new_sample()?)?;

  assert_eq!(file.objects.len(), 3);
  assert_eq!(file.objects[1].section, "m_stalker");
  assert_eq!(file.objects[1].name, "esc_wolf");
  assert_eq!(file.objects[1].position.x, 10.0);

  Ok(())
}

#[test]
fn counts_what_a_level_spawns_by_section() -> XrfResult {
  // A level spawns up to 2,832 objects out of at most 175 sections, so the sections are the answer and the objects
  // are not.
  let file: LevelSpawnFile = LevelSpawnFile::read_from_bytes::<XRayByteOrder>(new_sample()?)?;
  let sections = file.get_sections();

  assert_eq!(sections.len(), 2);
  assert_eq!(sections.get("graph_point"), Some(&2));
  assert_eq!(sections.get("m_stalker"), Some(&1));

  Ok(())
}

#[test]
fn keeps_the_rest_of_a_packet_verbatim() -> XrfResult {
  // Everything past the generic header is version- and class-dependent. It is carried rather than read, so a list
  // rewrites exactly without this reader having to resolve a server class.
  let file: LevelSpawnFile = LevelSpawnFile::read_from_bytes::<XRayByteOrder>(new_sample()?)?;

  assert_eq!(file.objects[0].payload, vec![7; 12]);
  assert_eq!(file.objects[1].payload.len(), 40);

  Ok(())
}

#[test]
fn writes_a_list_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = new_sample()?;
  let file: LevelSpawnFile = LevelSpawnFile::read_from_bytes::<XRayByteOrder>(original.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, original);

  Ok(())
}

#[test]
fn refuses_a_spawn_set_though_it_shares_the_extension() -> XrfResult {
  // A set opens with its header chunk, whose first field is the version and not `M_SPAWN`. Every shipped set reads
  // as action 10 here, which is what tells the two formats apart before either is parsed.
  let set: Vec<u8> = new_list(&[new_object(10, "", "", [0.0, 0.0, 0.0], &[0; 40])])?;

  assert!(LevelSpawnFile::read_from_bytes::<XRayByteOrder>(set).is_err());

  Ok(())
}
