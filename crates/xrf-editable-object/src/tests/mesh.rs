use xrf_chunk::{ChunkPayload, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::tests::fixtures;
use crate::{EditableMesh, EditableVMap, EditableVMapKind};

/// Reads mesh children the way the object's mesh list hands one over: inside a chunk of its own.
fn read(children: &[u8]) -> XrfResult<EditableMesh> {
  let mut reader: ChunkReader<InMemoryChunkDataSource> =
    ChunkReader::from_vec(fixtures::chunk(0, children))?.read_child_by_index(0)?;

  EditableMesh::read::<XRayByteOrder, _>(&mut reader)
}

fn write(mesh: &EditableMesh) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  mesh.write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

#[test]
fn a_mesh_reads_as_points_faces_and_the_maps_its_corners_use() -> XrfResult {
  let mesh: EditableMesh = read(&fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]))?;

  assert_eq!(mesh.name, "triangle");
  assert_eq!(mesh.points[1], Vector3d::new(1.0, 0.0, 0.0));
  assert_eq!(mesh.faces.len(), 1);
  assert_eq!(mesh.faces[0].corners[2].point, 2);
  assert_eq!(mesh.faces[0].corners[2].vmap_reference, 2);
  assert_eq!(mesh.smoothing_groups, Some(vec![1]));
  assert_eq!(mesh.vmap_references[1].points[0].index, 1);
  assert_eq!(mesh.surface_faces[0].surface, "default");
  assert_eq!(mesh.options, Some([0, 0]));
  assert!(!mesh.is_smoothing_group_masked());

  let vmap: &EditableVMap = &mesh.vmaps.as_ref().expect("expect the vertex maps")[0];

  assert_eq!(vmap.kind, EditableVMapKind::Uv);
  assert_eq!(vmap.get_count(), 3);
  assert_eq!(vmap.get_entry(2), Some([0.0, 1.0].as_slice()));
  assert_eq!(vmap.get_entry(3), None);
  assert_eq!(
    vmap.get_entry(usize::MAX),
    None,
    "an index no map reaches is none, not an overflow"
  );
  assert_eq!(vmap.polygon_indices, None);

  Ok(())
}

#[test]
fn a_mesh_is_written_back_byte_for_byte() -> XrfResult {
  for is_polymap in [false, true] {
    let original: Vec<u8> = fixtures::mesh_children(&fixtures::vmaps_payload(is_polymap), &[]);

    assert_eq!(write(&read(&original)?)?, original, "polymap {is_polymap}");
  }

  Ok(())
}

#[test]
fn a_mesh_another_exporter_laid_out_comes_back_in_its_own_order() -> XrfResult {
  // Mod-shipped objects store the vertex map references before the smoothing groups, the other way round from
  // `SaveMesh`; the editor finds both by id, and the file has to come back as it was.
  let mut chunks: Vec<ChunkPayload> = Vec::new();

  for chunk in ChunkReader::from_vec(fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]))
    .and_then(|mut reader| reader.read_children())
    .expect("expect the fixture to read")
  {
    let mut chunk = chunk;

    chunks.push(ChunkPayload {
      id: chunk.id,
      bytes: chunk.read_remaining().expect("expect the payload"),
    });
  }

  let groups: usize = chunks
    .iter()
    .position(|chunk| chunk.id == EditableMesh::SMOOTHING_GROUPS_CHUNK_ID)
    .expect("expect smoothing groups in the fixture");
  let moved: ChunkPayload = chunks.remove(groups);

  chunks.push(moved);

  let original: Vec<u8> = chunks
    .iter()
    .flat_map(|chunk| fixtures::chunk(chunk.id, &chunk.bytes))
    .collect();

  assert_eq!(write(&read(&original)?)?, original);

  Ok(())
}

#[test]
fn a_polygon_map_ties_each_entry_to_a_face() -> XrfResult {
  let mesh: EditableMesh = read(&fixtures::mesh_children(&fixtures::vmaps_payload(true), &[]))?;

  assert_eq!(
    mesh.vmaps.as_ref().expect("expect the vertex maps")[0].polygon_indices,
    Some(vec![0, 0, 0])
  );

  Ok(())
}

#[test]
fn a_polygon_map_mark_past_one_is_refused() {
  let mut vmaps: Vec<u8> = fixtures::vmaps_payload(false);

  // The count, the name and its terminator, then the dimension: the mark is the byte after.
  vmaps[4 + 8 + 1] = 2;

  let error: String = read(&fixtures::mesh_children(&vmaps, &[]))
    .expect_err("expect the mark refused")
    .to_string();

  assert!(error.contains("polygon map mark 2"), "Unexpected error: {error}");
}

#[test]
fn smoothing_groups_not_matching_the_faces_are_refused() {
  let mut children: Vec<u8> = Vec::new();

  // Rebuild the mesh with two groups for its one face.
  for chunk in ChunkReader::from_vec(fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]))
    .and_then(|mut reader| reader.read_children())
    .expect("expect the fixture to read")
  {
    let mut chunk = chunk;
    let payload: Vec<u8> = if chunk.id == EditableMesh::SMOOTHING_GROUPS_CHUNK_ID {
      fixtures::words(&[1, 2])
    } else {
      chunk.read_remaining().expect("expect the payload")
    };

    children.extend(fixtures::chunk(chunk.id, &payload));
  }

  let error: String = read(&children).expect_err("expect the groups refused").to_string();

  assert!(
    error.contains("2 smoothing groups for 1 faces"),
    "Unexpected error: {error}"
  );
}

#[test]
fn a_mesh_version_the_editor_does_not_load_is_refused() {
  let mut children: Vec<u8> = fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]);

  // The version is the first chunk's payload.
  children[8] = 0x10;

  let error: String = read(&children).expect_err("expect version 16 refused").to_string();

  assert!(error.contains("mesh version 16"), "Unexpected error: {error}");
}

#[test]
fn a_vertex_map_whose_lists_disagree_is_not_written() -> XrfResult {
  let mut mesh: EditableMesh = read(&fixtures::mesh_children(&fixtures::vmaps_payload(false), &[]))?;

  mesh.vmaps.as_mut().expect("expect the vertex maps")[0].values.pop();

  assert!(write(&mesh).is_err(), "no count describes 5 values over 3 entries of 2");

  Ok(())
}
