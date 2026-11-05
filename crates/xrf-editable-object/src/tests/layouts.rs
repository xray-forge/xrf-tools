//! The layouts older editors saved, which the CoP SDK 0.7's own `rawdata` still holds: each reduced from a file there
//! to the smallest bytes that carry it.

use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;

use crate::tests::fixtures;
use crate::{EditableMesh, EditableObjectFile, EditableTimeWidth};

/// A body with the version, the given flags payload and the given description payload.
fn object(flags: &[u8], description: &[u8]) -> Vec<u8> {
  let mut body: Vec<u8> = fixtures::chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x0010u16.to_le_bytes());

  body.extend(fixtures::chunk(EditableObjectFile::FLAGS_CHUNK_ID, flags));
  body.extend(fixtures::chunk(EditableObjectFile::DESCRIPTION_CHUNK_ID, description));

  fixtures::file(&body)
}

/// A description: creator, time, modifier, time, each time `width` bytes.
fn description(width: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = fixtures::string("\\\\YURIY\\Yuriy");

  bytes.extend(&1_242_207_820u64.to_le_bytes()[..width]);
  bytes.extend(fixtures::string("\\\\YURIY\\Yuriy"));
  bytes.extend(&1_242_207_821u64.to_le_bytes()[..width]);

  bytes
}

#[test]
fn a_description_with_64_bit_times_reads_and_comes_back() -> XrfResult {
  // 783 of the SDK's 1,762 objects, `objects\detail\det_hvosh.object` among them.
  for width in [EditableTimeWidth::Seconds32, EditableTimeWidth::Seconds64] {
    let original: Vec<u8> = object(&fixtures::words(&[0]), &description(width.get_size()));
    let object: EditableObjectFile = EditableObjectFile::read_from_bytes::<XRayByteOrder>(original.clone())?;
    let description = object.description.as_ref().expect("expect the description");

    assert_eq!(description.time_width, width);
    assert_eq!(description.modified_at, 1_242_207_821);
    assert_eq!(object.write_to_bytes::<XRayByteOrder>()?, original, "{width:?}");
  }

  Ok(())
}

#[test]
fn a_description_no_width_accounts_for_is_refused() {
  let mut bytes: Vec<u8> = description(8);

  bytes.push(0);

  assert!(EditableObjectFile::read_from_bytes::<XRayByteOrder>(object(&fixtures::words(&[0]), &bytes)).is_err());
}

#[test]
fn a_one_byte_flags_chunk_reads_as_r_chunk_reads_it() -> XrfResult {
  // `objects\editor\shadertest_sphere.object`, saved in 2008.
  let original: Vec<u8> = object(&[1], &description(4));
  let mut object: EditableObjectFile = EditableObjectFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

  assert!(object.is_dynamic());
  assert_eq!(object.flags_size, 1);
  assert_eq!(object.write_to_bytes::<XRayByteOrder>()?, original);

  object.flags = 0x100;

  assert!(
    object.write_to_bytes::<XRayByteOrder>().is_err(),
    "a flag past the stored byte has nowhere to go"
  );

  Ok(())
}

#[test]
fn an_empty_flags_chunk_is_refused() {
  assert!(EditableObjectFile::read_from_bytes::<XRayByteOrder>(object(&[], &description(4))).is_err());
}

/// The fixture mesh with its points chunk replaced.
fn mesh_with_points(points: &[u8]) -> XrfResult<EditableMesh> {
  let mut children: Vec<u8> = Vec::new();

  for mut chunk in
    ChunkReader::from_vec(fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]))?.read_children()?
  {
    let payload: Vec<u8> = if chunk.id == EditableMesh::POINTS_CHUNK_ID {
      points.to_vec()
    } else {
      chunk.read_remaining()?
    };

    children.extend(fixtures::chunk(chunk.id, &payload));
  }

  let mut reader: ChunkReader<InMemoryChunkDataSource> =
    ChunkReader::from_vec(fixtures::chunk(0, &children))?.read_child_by_index(0)?;

  EditableMesh::read::<XRayByteOrder, _>(&mut reader)
}

/// Three points, then where `faces` asks the faces around each: one count byte and the indices.
fn points_payload(faces: &[&[u32]]) -> Vec<u8> {
  let mut bytes: Vec<u8> = fixtures::words(&[3]);

  bytes.extend(fixtures::floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]));

  for list in faces {
    bytes.push(list.len() as u8);
    bytes.extend(fixtures::words(list));
  }

  bytes
}

/// The points chunk a mesh writes.
fn written_points(mesh: &EditableMesh) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  mesh.write::<XRayByteOrder>(&mut writer)?;

  let mut written: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(writer.buffer)?;

  ChunkReader::find_required_by_id(&written.read_children()?, EditableMesh::POINTS_CHUNK_ID)?.read_remaining()
}

#[test]
fn points_followed_by_their_faces_read_and_come_back() -> XrfResult {
  // 24 of the SDK's objects, `objects\editor\shadertest_plane.object` among them. The fixture's one face sits on all
  // three points.
  let payload: Vec<u8> = points_payload(&[&[0], &[0], &[0]]);
  let mesh: EditableMesh = mesh_with_points(&payload)?;

  assert!(mesh.has_point_faces);
  assert_eq!(mesh.get_point_faces()?, vec![vec![0], vec![0], vec![0]]);
  assert_eq!(written_points(&mesh)?, payload);

  Ok(())
}

#[test]
fn point_faces_follow_the_faces_an_edit_leaves() -> XrfResult {
  // The lists are derived on write, so a face added after reading shows in them rather than leaving them stale.
  let mut mesh: EditableMesh = mesh_with_points(&points_payload(&[&[0], &[0], &[0]]))?;

  mesh.faces.push(mesh.faces[0]);
  mesh
    .smoothing_groups
    .as_mut()
    .expect("expect the fixture's smoothing groups")
    .push(1);

  assert_eq!(written_points(&mesh)?, points_payload(&[&[0, 1], &[0, 1], &[0, 1]]));

  Ok(())
}

#[test]
fn stored_point_faces_the_faces_do_not_give_are_refused() {
  // Point 2 claims a face that is not there; writing would put down what the faces give instead.
  let error: String = mesh_with_points(&points_payload(&[&[0], &[0], &[1]]))
    .expect_err("expect stale lists refused")
    .to_string();

  assert!(error.contains("its faces do not give"), "Unexpected error: {error}");
}

#[test]
fn point_faces_not_one_list_per_point_are_refused() {
  // Two lists for three points: the third count byte runs past the chunk.
  assert!(mesh_with_points(&points_payload(&[&[0], &[0]])).is_err());
}
