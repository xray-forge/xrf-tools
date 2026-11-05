use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::tests::fixtures;
use crate::{EditableExtraChunk, EditableObjectFile};

fn read(bytes: Vec<u8>) -> XrfResult<EditableObjectFile> {
  EditableObjectFile::read_from_bytes::<XRayByteOrder>(bytes)
}

#[test]
fn an_object_reads_as_the_chunks_the_editor_saved() -> XrfResult {
  let object: EditableObjectFile = read(fixtures::dynamic_object())?;

  assert!(object.is_dynamic());
  assert_eq!(object.class_script.as_deref(), Some(""));
  assert_eq!(object.meshes.as_ref().map(Vec::len), Some(1));
  assert_eq!(object.bones.as_ref().map(Vec::len), Some(1));

  let surfaces = object.surfaces.as_ref().expect("expect the surfaces chunk");

  assert_eq!(surfaces[0].name, "default");
  assert_eq!(surfaces[0].shader, "models\\model");
  assert_eq!(surfaces[0].game_material, "default_object");
  assert_eq!(surfaces[0].texture_coordinate_count, 1);
  assert!(surfaces[0].is_two_sided());

  assert_eq!(
    object.motion_references,
    Some(vec![String::from("actors\\stalker"), String::from("actors\\idle")])
  );
  assert_eq!(
    object.bone_parts.as_ref().map(|parts| parts[0].bones.clone()),
    Some(vec![String::from("root")])
  );

  let description = object.description.as_ref().expect("expect the description chunk");

  assert_eq!(description.creator, "gsc");
  assert_eq!(description.modifier, "\\\\MAINPC\\gladi");
  assert_eq!(description.modified_at, 1_200_000_000);
  assert!(object.extra.is_empty());

  Ok(())
}

#[test]
fn an_object_is_written_back_byte_for_byte() -> XrfResult {
  let original: Vec<u8> = fixtures::dynamic_object();

  assert_eq!(read(original.clone())?.write_to_bytes::<XRayByteOrder>()?, original);

  Ok(())
}

#[test]
fn a_chunk_nothing_names_survives_at_the_end_of_every_container() -> XrfResult {
  // The writer puts kept chunks after the named ones, so a sentinel the editor left last comes back where it was.
  let original: Vec<u8> = fixtures::file(&fixtures::body_children(
    &fixtures::mesh_children(&fixtures::vmaps_payload(false), &fixtures::sentinel()),
    &fixtures::bone_children(true, &fixtures::sentinel()),
    &fixtures::sentinel(),
  ));
  let object: EditableObjectFile = read(original.clone())?;
  let sentinel = EditableExtraChunk {
    id: fixtures::SENTINEL_ID,
    bytes: vec![0xde, 0xad, 0xbe, 0xef],
  };

  assert_eq!(object.extra, vec![sentinel.clone()]);
  assert_eq!(
    object.meshes.as_ref().map(|meshes| meshes[0].extra.clone()),
    Some(vec![sentinel.clone()])
  );
  assert_eq!(
    object.bones.as_ref().map(|bones| bones[0].extra.clone()),
    Some(vec![sentinel])
  );
  assert_eq!(object.write_to_bytes::<XRayByteOrder>()?, original);

  Ok(())
}

#[test]
fn a_chunk_nothing_names_comes_back_where_the_file_stored_it() -> XrfResult {
  let mut body: Vec<u8> = fixtures::sentinel();

  body.extend(fixtures::body_children(
    &fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]),
    &fixtures::bone_children(true, &[]),
    &[],
  ));

  let original: Vec<u8> = fixtures::file(&body);
  let object: EditableObjectFile = read(original.clone())?;

  assert_eq!(object.extra.len(), 1);
  assert_eq!(object.write_to_bytes::<XRayByteOrder>()?, original);

  Ok(())
}

#[test]
fn an_object_built_in_code_is_written_in_the_editors_order() -> XrfResult {
  // Without a stored order the writer falls back to `CEditableObject::Save`'s, which the fixture follows.
  let original: Vec<u8> = fixtures::dynamic_object();
  let mut object: EditableObjectFile = read(original.clone())?;

  object.chunk_order.clear();

  for mesh in object.meshes.iter_mut().flatten() {
    mesh.chunk_order.clear();
  }

  for bone in object.bones.iter_mut().flatten() {
    bone.chunk_order.clear();
  }

  assert_eq!(object.write_to_bytes::<XRayByteOrder>()?, original);

  Ok(())
}

#[test]
fn a_named_chunk_added_after_reading_goes_where_the_editor_puts_it() -> XrfResult {
  let mut object: EditableObjectFile = read(fixtures::dynamic_object())?;

  object.embedded_motions = Some(vec![7]);

  let ids: Vec<u32> = chunk_ids(&object.write_to_bytes::<XRayByteOrder>()?)?;

  assert_eq!(
    ids
      .iter()
      .position(|id| *id == EditableObjectFile::EMBEDDED_MOTIONS_CHUNK_ID),
    Some(ids.len() - 1),
    "a chunk the stored order does not list follows the ones it does"
  );

  Ok(())
}

/// The body's chunk ids, in the order they sit in a file.
fn chunk_ids(bytes: &[u8]) -> XrfResult<Vec<u32>> {
  let mut root = xrf_chunk::ChunkReader::from_bytes(bytes)?;
  let mut body = root.read_child_by_index(0)?;

  Ok(body.read_children()?.iter().map(|chunk| chunk.id).collect())
}

#[test]
fn a_duplicate_of_a_named_chunk_is_kept_rather_than_read_over_the_first() -> XrfResult {
  // `IReader::find_chunk` stops at the first match, so the SDK reads the first and never sees the second.
  let mut body: Vec<u8> = fixtures::body_children(
    &fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]),
    &fixtures::bone_children(true, &[]),
    &[],
  );

  body.extend(fixtures::chunk(
    EditableObjectFile::LODS_CHUNK_ID,
    &fixtures::string("second"),
  ));

  let object: EditableObjectFile = read(fixtures::file(&body))?;

  assert_eq!(object.lods.as_deref(), Some(""));
  assert_eq!(object.extra[0].id, EditableObjectFile::LODS_CHUNK_ID);

  Ok(())
}

#[test]
fn a_version_the_editor_does_not_load_is_refused() -> XrfResult {
  let mut body: Vec<u8> = fixtures::chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x000fu16.to_le_bytes());

  body.extend(fixtures::chunk(
    EditableObjectFile::FLAGS_CHUNK_ID,
    &fixtures::words(&[0]),
  ));

  let error: String = read(fixtures::file(&body))
    .expect_err("expect version 15 refused")
    .to_string();

  assert!(error.contains("object version 15"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn an_object_without_flags_is_refused_as_the_editor_asserts() {
  let body: Vec<u8> = fixtures::chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x0010u16.to_le_bytes());
  let error: String = read(fixtures::file(&body))
    .expect_err("expect no flags refused")
    .to_string();

  assert!(error.contains("object flags"), "Unexpected error: {error}");
}

#[test]
fn a_root_holding_more_than_the_body_is_refused() {
  let mut bytes: Vec<u8> = fixtures::dynamic_object();

  bytes.extend(fixtures::sentinel());

  let error: String = read(bytes).expect_err("expect a second root chunk refused").to_string();

  assert!(error.contains("body chunk"), "Unexpected error: {error}");
}

#[test]
fn a_mesh_list_with_a_gap_is_refused() {
  // The editor opens mesh 0, 1, 2 and stops at the first missing, so mesh 1 here would vanish from it.
  let mesh: Vec<u8> = fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]);
  let mut meshes: Vec<u8> = fixtures::chunk(0, &mesh);

  meshes.extend(fixtures::chunk(2, &mesh));

  let mut body: Vec<u8> = fixtures::chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x0010u16.to_le_bytes());

  body.extend(fixtures::chunk(
    EditableObjectFile::FLAGS_CHUNK_ID,
    &fixtures::words(&[0]),
  ));
  body.extend(fixtures::chunk(EditableObjectFile::MESHES_CHUNK_ID, &meshes));

  let error: String = read(fixtures::file(&body))
    .expect_err("expect a gap refused")
    .to_string();

  assert!(
    error.contains("mesh chunk 2 at position 1"),
    "Unexpected error: {error}"
  );
}

#[test]
fn a_chunk_holding_more_than_its_fields_is_refused() {
  let mut surfaces: Vec<u8> = fixtures::surfaces_payload();

  surfaces.push(0);

  let mut body: Vec<u8> = fixtures::chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x0010u16.to_le_bytes());

  body.extend(fixtures::chunk(
    EditableObjectFile::FLAGS_CHUNK_ID,
    &fixtures::words(&[0]),
  ));
  body.extend(fixtures::chunk(EditableObjectFile::SURFACES_CHUNK_ID, &surfaces));

  assert!(
    read(fixtures::file(&body)).is_err(),
    "a byte past the surfaces would be dropped on the way back"
  );
}

#[test]
fn a_surface_count_past_its_chunk_is_refused_before_it_is_reserved() {
  let mut body: Vec<u8> = fixtures::chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x0010u16.to_le_bytes());

  body.extend(fixtures::chunk(
    EditableObjectFile::FLAGS_CHUNK_ID,
    &fixtures::words(&[0]),
  ));
  body.extend(fixtures::chunk(
    EditableObjectFile::SURFACES_CHUNK_ID,
    &fixtures::words(&[u32::MAX]),
  ));

  let error: String = read(fixtures::file(&body))
    .expect_err("expect the count refused")
    .to_string();

  assert!(error.contains("surfaces"), "Unexpected error: {error}");
}

#[test]
fn embedded_motions_are_kept_as_stored() -> XrfResult {
  let mut body: Vec<u8> = fixtures::chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x0010u16.to_le_bytes());

  body.extend(fixtures::chunk(
    EditableObjectFile::FLAGS_CHUNK_ID,
    &fixtures::words(&[1]),
  ));
  body.extend(fixtures::chunk(
    EditableObjectFile::EMBEDDED_MOTIONS_CHUNK_ID,
    &[1, 2, 3],
  ));

  let original: Vec<u8> = fixtures::file(&body);
  let object: EditableObjectFile = read(original.clone())?;

  assert_eq!(object.embedded_motions, Some(vec![1, 2, 3]));
  assert_eq!(object.write_to_bytes::<XRayByteOrder>()?, original);

  Ok(())
}

#[test]
fn an_object_written_to_a_path_reads_back_as_itself() -> XrfResult {
  let object: EditableObjectFile = read(fixtures::dynamic_object())?;
  let path = GeneratedResources::path("xrf-editable-object/round-trip/nested/object.object");

  object.write_to_path::<XRayByteOrder, _>(&path)?;

  assert_eq!(EditableObjectFile::read_from_path::<XRayByteOrder, _>(&path)?, object);

  Ok(())
}
