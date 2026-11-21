use xrf_chunk::ChunkWriter;
use xrf_error::XrfResult;

use crate::level::level_spawn_file::LevelSpawnFile;

fn new_chunk(id: u32, payload: &[u8]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.buffer = payload.to_vec();

  writer.flush_chunk_into_buffer(id)
}

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

// Numbered from zero, as every shipped file numbers them.
fn new_list(objects: &[Vec<u8>]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = Vec::new();

  for (index, object) in objects.iter().enumerate() {
    bytes.extend(new_chunk(index as u32, object)?);
  }

  Ok(bytes)
}

fn new_sample() -> XrfResult<Vec<u8>> {
  new_list(&[
    new_object(1, "graph_point", "fake_graph_point", [0.0, 0.0, 0.0], &[7; 12]),
    new_object(1, "m_stalker", "esc_wolf", [10.0, 1.0, -4.0], &[9; 40]),
    new_object(1, "graph_point", "fake_exit_point", [1.0, 0.0, 2.0], &[7; 12]),
  ])
}

#[test]
fn reads_every_object_of_a_flat_packet_run() -> XrfResult {
  let file: LevelSpawnFile = LevelSpawnFile::read_from_bytes(new_sample()?)?;

  assert_eq!(file.objects.len(), 3);
  assert_eq!(file.objects[1].section, "m_stalker");
  assert_eq!(file.objects[1].name, "esc_wolf");
  assert_eq!(file.objects[1].position.x, 10.0);

  Ok(())
}

#[test]
fn counts_what_a_level_spawns_by_section() -> XrfResult {
  // A level spawns up to 2,832 objects of at most 175 sections, so the sections are the answer.
  let file: LevelSpawnFile = LevelSpawnFile::read_from_bytes(new_sample()?)?;
  let sections = file.count_sections();

  assert_eq!(sections.len(), 2);
  assert_eq!(sections.get("graph_point"), Some(&2));
  assert_eq!(sections.get("m_stalker"), Some(&1));

  Ok(())
}

#[test]
fn keeps_the_rest_of_a_packet_verbatim() -> XrfResult {
  // The payload is carried, not read, so a list rewrites exactly without resolving a server class.
  let file: LevelSpawnFile = LevelSpawnFile::read_from_bytes(new_sample()?)?;

  assert_eq!(file.objects[0].payload, vec![7; 12]);
  assert_eq!(file.objects[1].payload.len(), 40);

  Ok(())
}

#[test]
fn writes_a_list_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = new_sample()?;
  let file: LevelSpawnFile = LevelSpawnFile::read_from_bytes(original.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write(&mut writer)?;

  assert_eq!(writer.flush_raw_into_buffer()?, original);

  Ok(())
}

#[test]
fn refuses_a_spawn_set_though_it_shares_the_extension() -> XrfResult {
  // A root spawn opens with its header, whose version every shipped set holds as 10 where `M_SPAWN` would be.
  let set: Vec<u8> = new_list(&[new_object(10, "", "", [0.0, 0.0, 0.0], &[0; 40])])?;

  assert!(LevelSpawnFile::read_from_bytes(set).is_err());

  Ok(())
}
