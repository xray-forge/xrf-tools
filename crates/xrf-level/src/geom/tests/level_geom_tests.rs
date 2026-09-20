use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;

use crate::geom::buffers::level_geom_vertex_element::LevelGeomVertexElement;
use crate::geom::level_geom_file::LevelGeomFile;
use crate::tests::fixtures::chunk;

/// One declaration element as the file stores it.
fn element(stream: u16, offset: u16, kind: u8, method: u8) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&stream.to_le_bytes());
  bytes.extend_from_slice(&offset.to_le_bytes());
  bytes.extend_from_slice(&[kind, method, 0, 0]);

  bytes
}

/// `D3DDECL_END`, which closes every declaration.
fn terminator() -> Vec<u8> {
  element(LevelGeomVertexElement::TERMINATOR_STREAM, 0, 17, 0)
}

/// A vertex buffer chunk holding one buffer of the given declaration and count.
fn vertex_buffers(declaration: &[Vec<u8>], vertices: u32, vertex_size: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = 1u32.to_le_bytes().to_vec();

  for item in declaration {
    bytes.extend_from_slice(item);
  }

  bytes.extend_from_slice(&terminator());
  bytes.extend_from_slice(&vertices.to_le_bytes());
  bytes.extend(std::iter::repeat_n(0u8, vertices as usize * vertex_size));

  bytes
}

/// An index buffer chunk holding one buffer of the given count.
fn index_buffers(indices: u32) -> Vec<u8> {
  let mut bytes: Vec<u8> = 1u32.to_le_bytes().to_vec();

  bytes.extend_from_slice(&indices.to_le_bytes());
  bytes.extend(std::iter::repeat_n(0u8, indices as usize * 2));

  bytes
}

/// A whole geometry file out of the chunks it is given.
fn geometry(vertices: &[u8], indices: &[u8], slide_windows: Option<&[u8]>) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID, vertices)?;

  bytes.extend(chunk(LevelGeomFile::INDEX_BUFFERS_CHUNK_ID, indices)?);

  if let Some(windows) = slide_windows {
    bytes.extend(chunk(LevelGeomFile::SLIDE_WINDOWS_CHUNK_ID, windows)?);
  }

  Ok(bytes)
}

#[test]
fn reads_a_buffer_shape_without_its_vertices() -> XrfResult {
  // Position as three floats and a colour as a packed word: twelve plus four.
  let declaration: Vec<Vec<u8>> = vec![element(0, 0, 2, 0), element(0, 12, 4, 0)];
  let file: LevelGeomFile = LevelGeomFile::read_from_bytes::<XRayByteOrder>(geometry(
    &vertex_buffers(&declaration, 4, 16),
    &index_buffers(6),
    None,
  )?)?;

  assert_eq!(file.vertex_buffers.len(), 1);
  assert_eq!(file.vertex_buffers[0].declaration.len(), 2);
  assert_eq!(file.vertex_buffers[0].get_vertex_size(), Some(16));
  assert_eq!(file.get_vertices_count(), 4);
  assert_eq!(file.get_indices_count(), 6);
  assert_eq!(file.get_triangles_count(), 2);

  Ok(())
}

#[test]
fn takes_a_vertex_size_from_the_furthest_element_rather_than_the_sum() -> XrfResult {
  // A declaration may leave a gap, so adding the element sizes up would under-measure the vertex and leave the read
  // short of the chunk's end. `FVF::ComputeVertexSize` takes the maximum of offset plus size instead.
  let declaration: Vec<Vec<u8>> = vec![element(0, 0, 2, 0), element(0, 16, 0, 0)];
  let file: LevelGeomFile = LevelGeomFile::read_from_bytes::<XRayByteOrder>(geometry(
    &vertex_buffers(&declaration, 2, 20),
    &index_buffers(3),
    None,
  )?)?;

  assert_eq!(file.vertex_buffers[0].get_vertex_size(), Some(20));

  Ok(())
}

#[test]
fn a_tessellated_element_occupies_no_vertex_bytes() -> XrfResult {
  // `D3DDECLMETHOD_UV` is generated rather than read, which `ComputeVertexSize` skips: counting it would make the
  // vertex bigger than the file wrote.
  let declaration: Vec<Vec<u8>> = vec![
    element(0, 0, 2, 0),
    element(0, 64, 3, LevelGeomVertexElement::METHOD_UV),
  ];
  let file: LevelGeomFile = LevelGeomFile::read_from_bytes::<XRayByteOrder>(geometry(
    &vertex_buffers(&declaration, 3, 12),
    &index_buffers(3),
    None,
  )?)?;

  assert_eq!(file.vertex_buffers[0].get_vertex_size(), Some(12));

  Ok(())
}

#[test]
fn an_element_of_a_second_stream_is_not_part_of_the_vertex() -> XrfResult {
  let declaration: Vec<Vec<u8>> = vec![element(0, 0, 2, 0), element(1, 0, 3, 0)];
  let file: LevelGeomFile = LevelGeomFile::read_from_bytes::<XRayByteOrder>(geometry(
    &vertex_buffers(&declaration, 2, 12),
    &index_buffers(3),
    None,
  )?)?;

  assert_eq!(file.vertex_buffers[0].get_vertex_size(), Some(12));

  Ok(())
}

#[test]
fn reads_the_detail_levels_of_a_progressive_mesh() -> XrfResult {
  let mut windows: Vec<u8> = 1u32.to_le_bytes().to_vec();

  windows.extend([0u8; 16]);
  windows.extend_from_slice(&2u32.to_le_bytes());
  windows.extend_from_slice(&0u32.to_le_bytes());
  windows.extend_from_slice(&120u16.to_le_bytes());
  windows.extend_from_slice(&64u16.to_le_bytes());
  windows.extend_from_slice(&360u32.to_le_bytes());
  windows.extend_from_slice(&40u16.to_le_bytes());
  windows.extend_from_slice(&24u16.to_le_bytes());

  let declaration: Vec<Vec<u8>> = vec![element(0, 0, 2, 0)];
  let file: LevelGeomFile = LevelGeomFile::read_from_bytes::<XRayByteOrder>(geometry(
    &vertex_buffers(&declaration, 1, 12),
    &index_buffers(3),
    Some(&windows),
  )?)?;

  assert_eq!(file.slide_windows.len(), 1);
  assert_eq!(file.get_detail_levels_count(), 2);
  assert_eq!(file.slide_windows[0].windows[0].triangles, 120);
  assert_eq!(file.slide_windows[0].windows[1].offset, 360);

  Ok(())
}

#[test]
fn detail_geometry_carries_no_progressive_meshes() -> XrfResult {
  // `level.geomX` is read by `LoadBuffers` alone; `LoadSWIs` is called on `level.geom` only, and the file has no
  // slide window chunk to find.
  let declaration: Vec<Vec<u8>> = vec![element(0, 0, 2, 0)];
  let file: LevelGeomFile = LevelGeomFile::read_from_bytes::<XRayByteOrder>(geometry(
    &vertex_buffers(&declaration, 1, 12),
    &index_buffers(3),
    None,
  )?)?;

  assert!(file.slide_windows.is_empty());

  Ok(())
}

#[test]
fn refuses_a_declaration_naming_a_type_direct3d_does_not() -> XrfResult {
  // Type 17 is `D3DDECLTYPE_UNUSED` and anything above it names nothing, so the vertex size is unanswerable. The
  // engine's own answer is a zero-length vertex, which would silently read the buffer as empty.
  let declaration: Vec<Vec<u8>> = vec![element(0, 0, 20, 0)];

  assert!(
    LevelGeomFile::read_from_bytes::<XRayByteOrder>(geometry(
      &vertex_buffers(&declaration, 1, 0),
      &index_buffers(3),
      None,
    )?)
    .is_err()
  );

  Ok(())
}

#[test]
fn refuses_a_buffer_reaching_past_the_chunk_holding_it() -> XrfResult {
  let mut vertices: Vec<u8> = 1u32.to_le_bytes().to_vec();

  vertices.extend(element(0, 0, 2, 0));
  vertices.extend(terminator());
  vertices.extend_from_slice(&64u32.to_le_bytes());

  assert!(LevelGeomFile::read_from_bytes::<XRayByteOrder>(geometry(&vertices, &index_buffers(3), None)?).is_err());

  Ok(())
}

#[test]
fn refuses_a_file_without_a_vertex_buffer_chunk() -> XrfResult {
  assert!(
    LevelGeomFile::read_from_bytes::<XRayByteOrder>(chunk(LevelGeomFile::INDEX_BUFFERS_CHUNK_ID, &index_buffers(3))?)
      .is_err()
  );

  Ok(())
}
