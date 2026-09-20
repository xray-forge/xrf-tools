use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::geom::level_geom_file::LevelGeomFile;
use crate::geom::level_geom_source::LevelGeomSource;
use crate::geom::level_geom_vertex_element::LevelGeomVertexElement;
use crate::geom::level_vertex::LevelVertex;
use crate::geom::level_vertex_layout::LevelVertexLayout;
use crate::tests::fixtures::chunk;

/// One declaration element as the file stores it: stream, offset, type, method, usage, usage index.
fn new_element(offset: u16, kind: u8, usage: u8, usage_index: u8) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&0u16.to_le_bytes());
  bytes.extend_from_slice(&offset.to_le_bytes());
  bytes.extend_from_slice(&[kind, 0, usage, usage_index]);

  bytes
}

/// `D3DDECL_END`.
fn new_terminator() -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&LevelGeomVertexElement::TERMINATOR_STREAM.to_le_bytes());
  bytes.extend_from_slice(&0u16.to_le_bytes());
  bytes.extend_from_slice(&[17, 0, 0, 0]);

  bytes
}

/// `r1_decl_lmap`, the declaration xrLC writes for a lightmapped surface.
fn new_lightmapped_declaration() -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&new_element(0, 2, 0, 0)); // POSITION0:FLOAT3
  bytes.extend_from_slice(&new_element(12, 4, 3, 0)); // NORMAL0:D3DCOLOR
  bytes.extend_from_slice(&new_element(16, 4, 6, 0)); // TANGENT0:D3DCOLOR
  bytes.extend_from_slice(&new_element(20, 4, 7, 0)); // BINORMAL0:D3DCOLOR
  bytes.extend_from_slice(&new_element(24, 6, 5, 0)); // TEXCOORD0:SHORT2
  bytes.extend_from_slice(&new_element(28, 6, 5, 1)); // TEXCOORD1:SHORT2
  bytes.extend_from_slice(&new_terminator());

  bytes
}

/// One vertex of that declaration, with values chosen so every field decodes to something recognisable.
fn new_lightmapped_vertex() -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&1.5f32.to_le_bytes());
  bytes.extend_from_slice(&2.5f32.to_le_bytes());
  bytes.extend_from_slice(&3.5f32.to_le_bytes());
  // Normal, written blue, green, red, alpha: x = 255, y = 128, z = 0, and hemi in the alpha.
  bytes.extend_from_slice(&[0, 128, 255, 77]);
  // Tangent and binormal: a direction in the same packing as the normal, whose alpha is the low byte of a base
  // coordinate rather than a component.
  bytes.extend_from_slice(&[0, 128, 255, 128]);
  bytes.extend_from_slice(&[255, 128, 0, 64]);
  // Base coordinate: one whole tile across, quantized by 1024.
  bytes.extend_from_slice(&1024i16.to_le_bytes());
  bytes.extend_from_slice(&512i16.to_le_bytes());
  // Lightmap coordinate: half the range, quantized by 32768.
  bytes.extend_from_slice(&16384i16.to_le_bytes());
  bytes.extend_from_slice(&(-16384i16).to_le_bytes());

  bytes
}

/// A whole `level.geom` holding one vertex buffer of the given vertices and one index buffer.
fn new_geometry(declaration: &[u8], vertices: &[u8], vertex_count: u32, indices: &[u16]) -> XrfResult<Vec<u8>> {
  let mut vertex_chunk: Vec<u8> = 1u32.to_le_bytes().to_vec();

  vertex_chunk.extend_from_slice(declaration);
  vertex_chunk.extend_from_slice(&vertex_count.to_le_bytes());
  vertex_chunk.extend_from_slice(vertices);

  let mut index_chunk: Vec<u8> = 1u32.to_le_bytes().to_vec();

  index_chunk.extend_from_slice(&(indices.len() as u32).to_le_bytes());

  for index in indices {
    index_chunk.extend_from_slice(&index.to_le_bytes());
  }

  let mut bytes: Vec<u8> = chunk(LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID, &vertex_chunk)?;

  bytes.extend(chunk(LevelGeomFile::INDEX_BUFFERS_CHUNK_ID, &index_chunk)?);

  Ok(bytes)
}

#[test]
fn test_reads_a_vertex_of_the_lightmapped_declaration() -> XrfResult {
  let source: LevelGeomSource<_> = LevelGeomSource::open_from_bytes::<XRayByteOrder>(new_geometry(
    &new_lightmapped_declaration(),
    &new_lightmapped_vertex(),
    1,
    &[0, 0, 0],
  )?)?;

  let vertices: Vec<LevelVertex> = source.read_vertices::<XRayByteOrder>(0, 0, 1)?;

  assert_eq!(vertices.len(), 1);

  let vertex: &LevelVertex = &vertices[0];

  assert_eq!(vertex.position.x, 1.5);
  assert_eq!(vertex.position.y, 2.5);
  assert_eq!(vertex.position.z, 3.5);
  assert!((vertex.normal.x - 1.0).abs() < 0.01, "normal x comes from the red byte");
  assert!(vertex.normal.z.abs() > 0.99, "normal z comes from the blue byte");
  assert_eq!(vertex.hemi, 77, "the normal's alpha is the hemisphere term");

  // 1024 / 1024 plus the tangent alpha's fraction, and 512 / 1024 plus the binormal's. Compared exactly, so the
  // divisor stays the engine's 255 - from `unpack_D3DCOLOR` normalizing the byte - rather than drifting to the
  // 255.5 the compiler wrote it with, which a loose tolerance would not notice.
  assert_eq!(
    vertex.texture_coordinate.0,
    (1024.0_f32 + 128.0_f32 / 255.0_f32) / LevelVertexLayout::BASE_QUANT
  );
  assert_eq!(
    vertex.texture_coordinate.1,
    (512.0_f32 + 64.0_f32 / 255.0_f32) / LevelVertexLayout::BASE_QUANT
  );
  assert_eq!(
    LevelVertexLayout::BASE_QUANT,
    1024.0,
    "32.0 / 32768.0 in the engine's shader"
  );
  assert_eq!(
    LevelVertexLayout::LIGHTMAP_QUANT,
    32768.0,
    "1.0 / 32768.0 in the engine's shader"
  );

  let lightmap: (f32, f32) = vertex.lightmap_coordinate.expect("a lightmapped vertex");

  assert!((lightmap.0 - 0.5).abs() < 1e-6);
  assert!((lightmap.1 + 0.5).abs() < 1e-6);
  assert_eq!(vertex.color, None, "a lightmapped surface carries no vertex colour");

  Ok(())
}

#[test]
fn test_reads_the_tangent_frame_beside_the_coordinate_its_alpha_carries() -> XrfResult {
  let source: LevelGeomSource<_> = LevelGeomSource::open_from_bytes::<XRayByteOrder>(new_geometry(
    &new_lightmapped_declaration(),
    &new_lightmapped_vertex(),
    1,
    &[0, 0, 0],
  )?)?;

  let vertices: Vec<LevelVertex> = source.read_vertices::<XRayByteOrder>(0, 0, 1)?;
  let tangent: &Vector3d = vertices[0].tangent.as_ref().expect("a declared tangent");
  let binormal: &Vector3d = vertices[0].binormal.as_ref().expect("a declared binormal");

  assert!((tangent.x - 1.0).abs() < 0.01, "tangent x comes from the red byte");
  assert!((tangent.z + 1.0).abs() < 0.01, "tangent z comes from the blue byte");
  assert!((binormal.x + 1.0).abs() < 0.01);
  assert!((binormal.z - 1.0).abs() < 0.01);

  Ok(())
}

#[test]
fn test_reads_a_range_from_the_middle_of_a_buffer() -> XrfResult {
  let mut vertices: Vec<u8> = Vec::new();

  for index in 0..4u32 {
    let mut vertex: Vec<u8> = new_lightmapped_vertex();

    vertex[0..4].copy_from_slice(&(index as f32).to_le_bytes());
    vertices.extend(vertex);
  }

  let source: LevelGeomSource<_> = LevelGeomSource::open_from_bytes::<XRayByteOrder>(new_geometry(
    &new_lightmapped_declaration(),
    &vertices,
    4,
    &[0, 1, 2, 3],
  )?)?;

  let read: Vec<LevelVertex> = source.read_vertices::<XRayByteOrder>(0, 2, 2)?;

  assert_eq!(read.len(), 2);
  assert_eq!(read[0].position.x, 2.0, "a base skips the vertices before it");
  assert_eq!(read[1].position.x, 3.0);

  assert_eq!(source.read_indices::<XRayByteOrder>(0, 1, 2)?, vec![1, 2]);

  Ok(())
}

#[test]
fn test_refuses_a_range_past_what_a_buffer_declares() -> XrfResult {
  let source: LevelGeomSource<_> = LevelGeomSource::open_from_bytes::<XRayByteOrder>(new_geometry(
    &new_lightmapped_declaration(),
    &new_lightmapped_vertex(),
    1,
    &[0, 0, 0],
  )?)?;

  assert!(
    source.read_vertices::<XRayByteOrder>(0, 0, 2).is_err(),
    "a count past the end is refused rather than reading a neighbour"
  );
  assert!(
    source.read_vertices::<XRayByteOrder>(0, 1, 1).is_err(),
    "a base at the end is refused too"
  );
  assert!(
    source.read_vertices::<XRayByteOrder>(1, 0, 1).is_err(),
    "a buffer no buffer answers to is refused"
  );
  assert!(source.read_indices::<XRayByteOrder>(0, 2, 2).is_err());

  Ok(())
}

#[test]
fn test_reads_the_fastpath_declaration_that_carries_positions_alone() -> XrfResult {
  let mut declaration: Vec<u8> = new_element(0, 2, 0, 0);

  declaration.extend(new_terminator());

  let source: LevelGeomSource<_> = LevelGeomSource::open_from_bytes::<XRayByteOrder>(new_geometry(
    &declaration,
    &[
      0, 0, 128, 63, // 1.0
      0, 0, 0, 64, // 2.0
      0, 0, 64, 64, // 3.0
    ],
    1,
    &[0],
  )?)?;

  let layout: LevelVertexLayout = LevelVertexLayout::of(&source.get_file().vertex_buffers[0])?;

  assert!(layout.is_fastpath());
  assert!(!layout.is_lightmapped());
  assert_eq!(layout.stride, 12);

  let vertices: Vec<LevelVertex> = source.read_vertices::<XRayByteOrder>(0, 0, 1)?;

  assert_eq!(vertices[0].position.x, 1.0);
  assert_eq!(vertices[0].lightmap_coordinate, None);

  Ok(())
}

#[test]
fn test_refuses_an_element_xrlc_does_not_write() -> XrfResult {
  let mut declaration: Vec<u8> = new_element(0, 2, 0, 0);

  // A blend weight, which level geometry never carries.
  declaration.extend(new_element(12, 2, 1, 0));
  declaration.extend(new_terminator());

  let source: XrfResult<LevelGeomSource<_>> =
    LevelGeomSource::open_from_bytes::<XRayByteOrder>(new_geometry(&declaration, &[0; 24], 1, &[0])?);
  let source: LevelGeomSource<_> = source?;

  assert!(
    source.read_vertices::<XRayByteOrder>(0, 0, 1).is_err(),
    "an unknown usage fails loudly rather than decoding the wrong bytes"
  );

  Ok(())
}

#[test]
fn test_tree_coordinates_use_the_tree_quantization() -> XrfResult {
  let mut declaration: Vec<u8> = new_element(0, 2, 0, 0);

  declaration.extend(new_element(12, 4, 3, 0)); // NORMAL0:D3DCOLOR
  declaration.extend(new_element(16, 7, 5, 0)); // TEXCOORD0:SHORT4
  declaration.extend(new_terminator());

  let mut vertex: Vec<u8> = vec![0; 16];

  vertex.extend_from_slice(&2048i16.to_le_bytes());
  vertex.extend_from_slice(&1024i16.to_le_bytes());
  vertex.extend_from_slice(&0i16.to_le_bytes());
  vertex.extend_from_slice(&0i16.to_le_bytes());

  let source: LevelGeomSource<_> =
    LevelGeomSource::open_from_bytes::<XRayByteOrder>(new_geometry(&declaration, &vertex, 1, &[0])?)?;

  let layout: LevelVertexLayout = LevelVertexLayout::of(&source.get_file().vertex_buffers[0])?;

  assert!(layout.is_tree());
  assert_eq!(layout.get_base_quant(), LevelVertexLayout::TREE_QUANT);

  let vertices: Vec<LevelVertex> = source.read_vertices::<XRayByteOrder>(0, 0, 1)?;

  assert!((vertices[0].texture_coordinate.0 - 1.0).abs() < 1e-6, "2048 / 2048");
  assert!((vertices[0].texture_coordinate.1 - 0.5).abs() < 1e-6, "1024 / 2048");

  Ok(())
}
