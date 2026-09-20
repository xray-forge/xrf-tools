use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::tests::fixtures::{chunk, floats, string};
use crate::wallmarks::level_wallmark_vertex::LevelWallmarkVertex;
use crate::wallmarks::level_wallmarks_file::LevelWallmarksFile;

/// One baked decal: a bounding sphere, a vertex count, then the lit vertices.
fn mark_bytes(vertices: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = floats(&[1.0, 2.0, 3.0, 0.5]);

  bytes.extend_from_slice(&(vertices as u32).to_le_bytes());

  for index in 0..vertices {
    bytes.extend(floats(&[index as f32, 0.0, 0.0]));
    bytes.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    bytes.extend(floats(&[0.25, 0.75]));
  }

  bytes
}

/// One slot: a decal count, then its names and decals where it holds any.
fn slot_bytes(shader: &str, texture: &str, marks: &[usize]) -> Vec<u8> {
  let mut bytes: Vec<u8> = (marks.len() as u32).to_le_bytes().to_vec();

  if marks.is_empty() {
    return bytes;
  }

  bytes.extend(string(shader));
  bytes.extend(string(texture));

  for vertices in marks {
    bytes.extend(mark_bytes(*vertices));
  }

  bytes
}

/// A whole file: one chunk holding a slot count and the slots.
fn file_bytes(slots: &[(&str, &str, Vec<usize>)]) -> XrfResult<Vec<u8>> {
  let mut payload: Vec<u8> = (slots.len() as u32).to_le_bytes().to_vec();

  for (shader, texture, marks) in slots {
    payload.extend(slot_bytes(shader, texture, marks));
  }

  chunk(LevelWallmarksFile::SLOTS_CHUNK_ID, &payload)
}

#[test]
fn a_wallmark_vertex_is_the_width_its_lit_layout_gives_it() {
  assert_eq!(LevelWallmarkVertex::SERIALIZED_SIZE, 24);
}

#[test]
fn a_slot_holds_every_decal_that_shares_its_material() -> XrfResult {
  let file: LevelWallmarksFile = LevelWallmarksFile::read_from_bytes::<XRayByteOrder>(file_bytes(&[
    ("effects\\wallmark", "wm\\wm_blood", vec![4, 6]),
    ("effects\\wallmark", "wm\\wm_scorch", vec![3]),
  ])?)?;

  assert_eq!(file.slots.len(), 2);
  assert_eq!(file.slots[0].texture, "wm\\wm_blood");
  assert_eq!(file.get_marks_count(), 3);
  assert_eq!(file.get_vertices_count(), 13);
  // A fan of four vertices draws two triangles.
  assert_eq!(file.slots[0].marks[0].get_triangles_count(), 2);
  assert_eq!(file.slots[0].marks[0].radius, 0.5);

  Ok(())
}

#[test]
fn a_slot_holding_nothing_stores_no_names_at_all() -> XrfResult {
  // `ESceneWallmarkTool::Export` writes the names only inside the non-empty branch, so a reader that always read
  // them would run straight into the next slot.
  let bytes: Vec<u8> = file_bytes(&[("", "", vec![]), ("effects\\wallmark", "wm\\wm_blood", vec![4])])?;

  let file: LevelWallmarksFile = LevelWallmarksFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  assert_eq!(file.slots.len(), 2);
  assert!(!file.slots[0].is_used());
  assert!(file.slots[1].is_used());
  assert_eq!(file.slots[1].texture, "wm\\wm_blood");

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_decal_list_is_written_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&[
    ("effects\\wallmark", "wm\\wm_blood", vec![4, 6]),
    ("effects\\wallmark", "wm\\wm_scorch", vec![3]),
  ])?;

  let file: LevelWallmarksFile = LevelWallmarksFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_file_carrying_no_slots_reads_as_none_rather_than_failing() -> XrfResult {
  let file: LevelWallmarksFile = LevelWallmarksFile::read_from_bytes::<XRayByteOrder>(file_bytes(&[])?)?;

  assert!(file.slots.is_empty());
  assert_eq!(file.get_marks_count(), 0);

  Ok(())
}
