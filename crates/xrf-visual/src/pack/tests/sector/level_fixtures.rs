//! Synthetic compiled levels built in code, so the bytes a sector test reasons about are visible in the test.

use xrf_chunk::XRayByteOrder;
use xrf_level::{
  LevelFile, LevelGeomFile, LevelGeomSource, LevelGeomVertexElement, LevelShadersChunk, LevelVisualsChunk,
};
use xrf_ogf::{OgfChildrenLinkChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfTreeDefinitionChunk};

/// One chunk as a file stores it: its identifier, its length, and its payload.
pub(crate) fn new_chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = id.to_le_bytes().to_vec();

  bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

/// One declaration element: stream, offset, type, method, usage, usage index.
fn new_element(offset: u16, kind: u8, usage: u8, usage_index: u8) -> Vec<u8> {
  let mut bytes: Vec<u8> = 0u16.to_le_bytes().to_vec();

  bytes.extend_from_slice(&offset.to_le_bytes());
  bytes.extend_from_slice(&[kind, 0, usage, usage_index]);

  bytes
}

/// `D3DDECL_END`, which ends a declaration rather than describing an element.
fn new_terminator() -> Vec<u8> {
  let mut bytes: Vec<u8> = LevelGeomVertexElement::TERMINATOR_STREAM.to_le_bytes().to_vec();

  bytes.extend_from_slice(&0u16.to_le_bytes());
  bytes.extend_from_slice(&[17, 0, 0, 0]);

  bytes
}

/// `r1_decl_lmap`, the declaration xrLC writes for a lightmapped surface.
pub(crate) fn new_lightmapped_declaration() -> Vec<u8> {
  let mut bytes: Vec<u8> = new_element(0, 2, 0, 0);

  bytes.extend(new_element(12, 4, 3, 0));
  bytes.extend(new_element(16, 4, 6, 0));
  bytes.extend(new_element(20, 4, 7, 0));
  bytes.extend(new_element(24, 6, 5, 0));
  bytes.extend(new_element(28, 6, 5, 1));
  bytes.extend(new_terminator());

  bytes
}

/// The declaration that carries a position and nothing else, which is what a fast path stores.
pub(crate) fn new_positions_declaration() -> Vec<u8> {
  let mut bytes: Vec<u8> = new_element(0, 2, 0, 0);

  bytes.extend(new_terminator());

  bytes
}

/// One lightmapped vertex, whose every field decodes to something the test can recognise.
pub(crate) fn new_lightmapped_vertex(x: f32, y: f32, z: f32) -> Vec<u8> {
  let mut bytes: Vec<u8> = x.to_le_bytes().to_vec();

  bytes.extend_from_slice(&y.to_le_bytes());
  bytes.extend_from_slice(&z.to_le_bytes());
  // Normal, tangent and binormal, written blue, green, red, alpha.
  bytes.extend_from_slice(&[0, 128, 255, 77]);
  bytes.extend_from_slice(&[0, 128, 255, 0]);
  bytes.extend_from_slice(&[255, 128, 0, 0]);
  // One whole tile across, and half a lightmap.
  bytes.extend_from_slice(&1024i16.to_le_bytes());
  bytes.extend_from_slice(&512i16.to_le_bytes());
  bytes.extend_from_slice(&16384i16.to_le_bytes());
  bytes.extend_from_slice(&(-16384i16).to_le_bytes());

  bytes
}

/// One vertex of the positions-only declaration.
pub(crate) fn new_position_vertex(x: f32, y: f32, z: f32) -> Vec<u8> {
  let mut bytes: Vec<u8> = x.to_le_bytes().to_vec();

  bytes.extend_from_slice(&y.to_le_bytes());
  bytes.extend_from_slice(&z.to_le_bytes());

  bytes
}

/// One vertex buffer of a level's render geometry: what a vertex is made of, and the vertices themselves.
pub(crate) struct GeomBuffer {
  pub declaration: Vec<u8>,
  pub vertices: Vec<Vec<u8>>,
}

/// A whole `level.geom` of the given vertex buffers and one index buffer.
pub(crate) fn new_geometry_fixture(buffers: &[GeomBuffer], indices: &[u16]) -> Vec<u8> {
  let mut vertex_chunk: Vec<u8> = (buffers.len() as u32).to_le_bytes().to_vec();

  for buffer in buffers {
    vertex_chunk.extend_from_slice(&buffer.declaration);
    vertex_chunk.extend_from_slice(&(buffer.vertices.len() as u32).to_le_bytes());

    for vertex in &buffer.vertices {
      vertex_chunk.extend_from_slice(vertex);
    }
  }

  let mut index_chunk: Vec<u8> = 1u32.to_le_bytes().to_vec();

  index_chunk.extend_from_slice(&(indices.len() as u32).to_le_bytes());

  for index in indices {
    index_chunk.extend_from_slice(&index.to_le_bytes());
  }

  let mut bytes: Vec<u8> = new_chunk(LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID, &vertex_chunk);

  bytes.extend(new_chunk(LevelGeomFile::INDEX_BUFFERS_CHUNK_ID, &index_chunk));

  bytes
}

/// Opens render geometry the same way the tool does, from bytes already in hand.
pub(crate) fn new_open_geometry(bytes: Vec<u8>) -> LevelGeomSource<xrf_chunk::InMemoryChunkDataSource> {
  LevelGeomSource::open_from_bytes::<XRayByteOrder>(bytes).expect("readable render geometry")
}

/// An OGF header of the given model type and shader entry.
fn new_header(model_type: u8, shader_id: u16) -> Vec<u8> {
  new_header_at(model_type, shader_id, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0])
}

/// The same, declaring where in the level the visual says it is.
fn new_header_at(model_type: u8, shader_id: u16, min: [f32; 3], max: [f32; 3]) -> Vec<u8> {
  let mut bytes: Vec<u8> = vec![4, model_type];

  bytes.extend_from_slice(&shader_id.to_le_bytes());

  for value in [min[0], min[1], min[2], max[0], max[1], max[2], 0.0, 0.0, 0.0, 1.0] {
    bytes.extend_from_slice(&value.to_le_bytes());
  }

  new_chunk(OgfHeaderChunk::CHUNK_ID, &bytes)
}

/// A drawable declaring the extent it occupies in the level.
pub(crate) fn new_drawable_at(shader_id: u16, min: [f32; 3], max: [f32; 3]) -> Vec<u8> {
  let mut bytes: Vec<u8> = new_header_at(0, shader_id, min, max);
  let mut container: Vec<u8> = Vec::new();

  for value in [0u32, 0, 2, 0, 0, 3] {
    container.extend_from_slice(&value.to_le_bytes());
  }

  bytes.extend(new_chunk(OgfGeometryContainerChunk::CHUNK_ID, &container));

  bytes
}

/// A visual that draws the given range of the level's shared buffers.
pub(crate) fn new_drawable(
  shader_id: u16,
  vertex_base: u32,
  vertex_count: u32,
  index_base: u32,
  index_count: u32,
) -> Vec<u8> {
  new_drawable_of_buffer(shader_id, 0, vertex_base, vertex_count, index_base, index_count)
}

/// The same, drawing out of a named vertex buffer rather than the first.
pub(crate) fn new_drawable_of_buffer(
  shader_id: u16,
  vertex_buffer: u32,
  vertex_base: u32,
  vertex_count: u32,
  index_base: u32,
  index_count: u32,
) -> Vec<u8> {
  let mut bytes: Vec<u8> = new_header(0, shader_id);
  let mut container: Vec<u8> = Vec::new();

  for value in [vertex_buffer, vertex_base, vertex_count, 0, index_base, index_count] {
    container.extend_from_slice(&value.to_le_bytes());
  }

  bytes.extend(new_chunk(OgfGeometryContainerChunk::CHUNK_ID, &container));

  bytes
}

/// A visual that draws nothing itself and links the visuals that do.
pub(crate) fn new_hierarchy(children: &[u32]) -> Vec<u8> {
  let mut bytes: Vec<u8> = new_header(1, 0);
  let mut links: Vec<u8> = (children.len() as u32).to_le_bytes().to_vec();

  for child in children {
    links.extend_from_slice(&child.to_le_bytes());
  }

  bytes.extend(new_chunk(OgfChildrenLinkChunk::CHUNK_ID, &links));

  bytes
}

/// A visuals run of the given visuals, each already framed as its own numbered chunk payload.
pub(crate) fn new_visuals(run: &[Vec<u8>]) -> LevelVisualsChunk {
  let mut body: Vec<u8> = Vec::new();

  for (index, visual) in run.iter().enumerate() {
    body.extend(new_chunk(index as u32, visual));
  }

  let level: Vec<u8> = new_chunk(LevelVisualsChunk::CHUNK_ID, &body);

  LevelFile::read_visuals_from_bytes::<XRayByteOrder>(level)
    .expect("readable visuals")
    .expect("a run of visuals")
}

/// A shader table of the given raw entries, as the level stores them.
pub(crate) fn new_shaders(entries: &[&str]) -> LevelShadersChunk {
  LevelShadersChunk {
    entries: entries
      .iter()
      .map(|entry| xrf_level::LevelShaderEntry::parse(entry))
      .collect(),
  }
}

/// A tree: the same range of the shared buffers, stood where its own transform puts it.
pub(crate) fn new_tree(shader_id: u16, vertex_base: u32, vertex_count: u32, index_count: u32, at: f32) -> Vec<u8> {
  new_lit_tree(shader_id, vertex_base, vertex_count, index_count, at, [0.0, 0.0])
}

/// A tree whose colour terms carry a hemisphere scale and bias, as the file stores them before the engine halves both.
pub(crate) fn new_lit_tree(
  shader_id: u16,
  vertex_base: u32,
  vertex_count: u32,
  index_count: u32,
  at: f32,
  hemi: [f32; 2],
) -> Vec<u8> {
  let mut bytes: Vec<u8> = new_drawable_of_buffer(shader_id, 0, vertex_base, vertex_count, 0, index_count);
  let mut definition: Vec<u8> = Vec::new();

  // Row major, translation in the fourth row, as the engine stores one.
  for value in [
    1.0f32, 0.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, 0.0, //
    0.0, 0.0, 1.0, 0.0, //
    at, 0.0, 0.0, 1.0,
  ] {
    definition.extend_from_slice(&value.to_le_bytes());
  }

  // Scale and bias, five floats each: colour, then hemisphere, then sun.
  for value in [0.0f32, 0.0, 0.0, hemi[0], 0.0, 0.0, 0.0, 0.0, hemi[1], 0.0] {
    definition.extend_from_slice(&value.to_le_bytes());
  }

  bytes.extend(new_chunk(OgfTreeDefinitionChunk::CHUNK_ID, &definition));

  bytes
}

/// `r1_decl_vert`, the declaration xrLC writes for a vertex lit surface: a baked colour in place of a lightmap.
pub(crate) fn new_vertex_lit_declaration() -> Vec<u8> {
  let mut bytes: Vec<u8> = new_element(0, 2, 0, 0);

  bytes.extend(new_element(12, 4, 3, 0));
  bytes.extend(new_element(16, 4, 10, 0));
  bytes.extend(new_element(20, 6, 5, 0));
  bytes.extend(new_terminator());

  bytes
}

/// One vertex lit vertex, carrying the colour the compiler baked into it.
pub(crate) fn new_vertex_lit_vertex(x: f32, y: f32, z: f32, color: [u8; 4]) -> Vec<u8> {
  let mut bytes: Vec<u8> = x.to_le_bytes().to_vec();

  bytes.extend_from_slice(&y.to_le_bytes());
  bytes.extend_from_slice(&z.to_le_bytes());
  // Normal, written blue, green, red, alpha.
  bytes.extend_from_slice(&[0, 128, 255, 77]);
  bytes.extend_from_slice(&color);
  bytes.extend_from_slice(&1024i16.to_le_bytes());
  bytes.extend_from_slice(&512i16.to_le_bytes());

  bytes
}

/// The declaration xrLC writes for a tree: the tangent frame, and a coordinate of four shorts whose last two are wind.
pub(crate) fn new_tree_declaration() -> Vec<u8> {
  let mut bytes: Vec<u8> = new_element(0, 2, 0, 0);

  bytes.extend(new_element(12, 4, 3, 0));
  bytes.extend(new_element(16, 4, 6, 0));
  bytes.extend(new_element(20, 4, 7, 0));
  bytes.extend(new_element(24, 7, 5, 0));
  bytes.extend(new_terminator());

  bytes
}

/// One tree vertex: a coordinate of 2048 and 1024 over the tree's 2048, then wind terms 7 and 9.
pub(crate) fn new_tree_vertex(x: f32, y: f32, z: f32) -> Vec<u8> {
  let mut bytes: Vec<u8> = x.to_le_bytes().to_vec();

  bytes.extend_from_slice(&y.to_le_bytes());
  bytes.extend_from_slice(&z.to_le_bytes());
  bytes.extend_from_slice(&[0, 128, 255, 77]);
  bytes.extend_from_slice(&[0, 128, 255, 200]);
  bytes.extend_from_slice(&[255, 128, 0, 100]);

  for value in [2048i16, 1024, 7, 9] {
    bytes.extend_from_slice(&value.to_le_bytes());
  }

  bytes
}
