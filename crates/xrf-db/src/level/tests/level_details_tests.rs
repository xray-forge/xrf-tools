use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::data::detail::detail_model::DetailModel;
use crate::level::level_details_file::{DETAIL_SLOT_METERS, LevelDetailsFile, LevelDetailsHeader};
use crate::level::level_details_slot::LevelDetailsSlot;

/// One detail object, as `CDetail::Load` lays it out.
fn model_bytes(shader: &str, texture: &str, flags: u32, vertices: u32, indices: u32) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(shader.as_bytes());
  bytes.push(0);
  bytes.extend_from_slice(texture.as_bytes());
  bytes.push(0);
  bytes.extend_from_slice(&flags.to_le_bytes());
  bytes.extend_from_slice(&0.5f32.to_le_bytes());
  bytes.extend_from_slice(&1.5f32.to_le_bytes());
  bytes.extend_from_slice(&vertices.to_le_bytes());
  bytes.extend_from_slice(&indices.to_le_bytes());

  for index in 0..vertices {
    for component in [index as f32, index as f32 * 2.0, index as f32 * 3.0, 0.25, 0.75] {
      bytes.extend_from_slice(&component.to_le_bytes());
    }
  }

  for index in 0..indices {
    bytes.extend_from_slice(&((index % vertices.max(1)) as u16).to_le_bytes());
  }

  bytes
}

/// A slot's stored word followed by its untouched palette.
fn slot_bytes(objects: [u64; 4]) -> Vec<u8> {
  let word: u64 = (objects[0] << 20) | (objects[1] << 26) | (objects[2] << 32) | (objects[3] << 38);
  let mut bytes: Vec<u8> = word.to_le_bytes().to_vec();

  bytes.extend_from_slice(&[0; 8]);

  bytes
}

/// A whole library: a header, two numbered objects and a two by two grid.
///
/// Built by hand rather than by writing a value back out, so the test pins the layout the engine reads rather than
/// this reader agreeing with itself.
fn library_bytes() -> XrfResult<Vec<u8>> {
  let mut header: ChunkWriter = ChunkWriter::new();

  for value in [3u32, 2, 0, 0, 2, 2] {
    header.buffer.extend_from_slice(&value.to_le_bytes());
  }

  let mut objects: ChunkWriter = ChunkWriter::new();

  for (index, bytes) in [
    model_bytes("details\\blend", "detail\\grass", 0, 4, 6),
    model_bytes("details\\blend", "detail\\stone", 1, 3, 3),
  ]
  .into_iter()
  .enumerate()
  {
    let mut entry: ChunkWriter = ChunkWriter::new();

    entry.buffer = bytes;
    entry.flush_chunk_into::<XRayByteOrder>(&mut objects.buffer, index as u32)?;
  }

  let mut slots: ChunkWriter = ChunkWriter::new();

  // Three planted corners over four slots: object 0 twice, object 1 once, and one empty slot.
  for objects in [[0, 0x3F, 0x3F, 0x3F], [0, 1, 0x3F, 0x3F], [0x3F; 4], [0x3F; 4]] {
    slots.buffer.extend(slot_bytes(objects));
  }

  // Objects, grid, then header - the order every shipped library uses, because the object count is only known once
  // the objects are written.
  let mut bytes: Vec<u8> = objects.flush_chunk_into_buffer::<XRayByteOrder>(LevelDetailsFile::OBJECTS_CHUNK_ID)?;

  bytes.extend(slots.flush_chunk_into_buffer::<XRayByteOrder>(LevelDetailsFile::SLOTS_CHUNK_ID)?);
  bytes.extend(header.flush_chunk_into_buffer::<XRayByteOrder>(LevelDetailsFile::HEADER_CHUNK_ID)?);

  Ok(bytes)
}

#[test]
fn a_library_reads_as_the_header_objects_and_grid_it_is() -> XrfResult {
  let library: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(library_bytes()?)?;

  assert_eq!(library.header.version, LevelDetailsFile::CURRENT_VERSION);
  assert_eq!(library.header.object_count, 2);
  assert_eq!((library.header.size_x, library.header.size_z), (2, 2));
  assert_eq!(library.objects.len(), 2);
  assert_eq!(library.objects[0].texture, "detail\\grass");
  assert_eq!(library.objects[0].vertices.len(), 4);
  assert_eq!(library.objects[0].indices.len(), 6);

  Ok(())
}

#[test]
fn a_library_is_written_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = library_bytes()?;
  let library: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  library.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, original);

  Ok(())
}

#[test]
fn a_grid_reports_what_it_plants_rather_than_how_many_cells_it_has() -> XrfResult {
  let library: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(library_bytes()?)?;

  assert_eq!(library.get_slots_count(), 4);
  assert_eq!(library.get_planted_slots_count(), 2);
  // Corners, not slots: object 0 sits in two of them and object 1 in one.
  assert_eq!(library.get_object_usage(), vec![2, 1]);

  Ok(())
}

#[test]
fn a_grid_covers_the_ground_its_slot_size_gives_it() -> XrfResult {
  let library: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(library_bytes()?)?;

  assert_eq!(
    library.get_covered_meters(),
    (2.0 * DETAIL_SLOT_METERS, 2.0 * DETAIL_SLOT_METERS)
  );

  Ok(())
}

#[test]
fn a_slot_is_decoded_only_where_one_is_looked_at() -> XrfResult {
  let library: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(library_bytes()?)?;
  let slots: Vec<LevelDetailsSlot> = library.iter_slots().collect();

  assert_eq!(slots.len(), 4);
  assert_eq!(slots[1].objects, [Some(0), Some(1), None, None]);
  assert!(!slots[3].is_planted());

  Ok(())
}

#[test]
fn a_model_says_whether_the_renderer_sways_it() -> XrfResult {
  let library: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(library_bytes()?)?;

  // The flag turns swaying off, so a clear word is the one that waves.
  assert!(library.objects[0].is_waving());
  assert!(!library.objects[1].is_waving());
  assert_eq!(library.objects[0].get_triangles_count(), 2);

  Ok(())
}

#[test]
fn a_model_reports_the_box_its_mesh_occupies() -> XrfResult {
  let library: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(library_bytes()?)?;
  let (minimum, maximum) = library.objects[0].get_bounds().expect("a keyed mesh has bounds");

  assert_eq!((minimum.x, minimum.y, minimum.z), (0.0, 0.0, 0.0));
  assert_eq!((maximum.x, maximum.y, maximum.z), (3.0, 6.0, 9.0));

  Ok(())
}

#[test]
fn a_model_carrying_no_mesh_has_no_bounds_rather_than_an_empty_box() -> XrfResult {
  let mut library: LevelDetailsFile = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(library_bytes()?)?;

  library.objects[0].vertices.clear();
  library.objects[0].indices.clear();

  assert_eq!(library.objects[0].get_bounds(), None);

  Ok(())
}

/// Where a field of the trailing header sits, counted back from the end of the file.
fn header_field(bytes: &[u8], index: usize) -> std::ops::Range<usize> {
  let start: usize = bytes.len() - LevelDetailsHeader::SIZE as usize + index * 4;

  start..start + 4
}

#[test]
fn a_version_nothing_here_reads_is_refused_by_name() -> XrfResult {
  let mut bytes: Vec<u8> = library_bytes()?;

  let version: std::ops::Range<usize> = header_field(&bytes, 0);

  bytes[version].copy_from_slice(&7u32.to_le_bytes());

  let error: String = LevelDetailsFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("an unknown version is refused")
    .to_string();

  assert!(error.contains('7'), "'{error}' names the version it refused");

  Ok(())
}

#[test]
fn a_grid_the_header_does_not_account_for_is_refused() -> XrfResult {
  let mut bytes: Vec<u8> = library_bytes()?;

  // A three by two grid over a payload holding two by two.
  let size_x: std::ops::Range<usize> = header_field(&bytes, 4);

  bytes[size_x].copy_from_slice(&3u32.to_le_bytes());

  assert!(LevelDetailsFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

#[test]
fn an_index_count_that_is_not_whole_triangles_is_refused() {
  let mut bytes: Vec<u8> = model_bytes("details\\blend", "detail\\grass", 0, 4, 6);

  // The index count sits after the two names, the flags and the two scales.
  let offset: usize = "details\\blend".len() + 1 + "detail\\grass".len() + 1 + 4 + 4 + 4 + 4;

  bytes[offset..offset + 4].copy_from_slice(&4u32.to_le_bytes());

  let error: String = DetailModel::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("a partial triangle is refused")
    .to_string();

  assert!(error.contains("whole triangles"), "Unexpected error: {error}");
}

#[test]
fn a_standalone_model_is_the_same_layout_as_a_library_entry() -> XrfResult {
  // `model_CreateDM` is `CDetail::Load` over the whole file, so a `.dm` and an entry read identically.
  let bytes: Vec<u8> = model_bytes("effects\\rain", "fx\\fx_rain", 1, 2, 3);
  let model: DetailModel = DetailModel::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  assert_eq!(model.shader, "effects\\rain");
  assert_eq!(model.texture, "fx\\fx_rain");
  assert_eq!(model.vertices.len(), 2);
  assert_eq!(model.get_triangles_count(), 1);

  let mut writer: ChunkWriter = ChunkWriter::new();

  model.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_standalone_model_with_anything_after_it_is_refused() {
  let mut bytes: Vec<u8> = model_bytes("effects\\rain", "fx\\fx_rain", 1, 2, 3);

  bytes.extend_from_slice(&[0, 0, 0, 0]);

  assert!(DetailModel::read_from_bytes::<XRayByteOrder>(bytes).is_err());
}
