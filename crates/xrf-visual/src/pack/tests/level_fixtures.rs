//! Synthetic compiled levels built in code, so the bytes a sector test reasons about are visible in the test.

use xrf_chunk::XRayByteOrder;
use xrf_level::{
  LevelFile, LevelGeomFile, LevelGeomSource, LevelGeomVertexElement, LevelShadersChunk, LevelVisualsChunk,
};
use xrf_ogf::{OgfChildrenLinkChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfTreeDefinitionChunk};

/// One chunk as a file stores it: its identifier, its length, and its payload.
pub(crate) fn chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = id.to_le_bytes().to_vec();

  bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

/// One declaration element: stream, offset, type, method, usage, usage index.
fn element(offset: u16, kind: u8, usage: u8, usage_index: u8) -> Vec<u8> {
  let mut bytes: Vec<u8> = 0u16.to_le_bytes().to_vec();

  bytes.extend_from_slice(&offset.to_le_bytes());
  bytes.extend_from_slice(&[kind, 0, usage, usage_index]);

  bytes
}

/// `D3DDECL_END`, which ends a declaration rather than describing an element.
fn terminator() -> Vec<u8> {
  let mut bytes: Vec<u8> = LevelGeomVertexElement::TERMINATOR_STREAM.to_le_bytes().to_vec();

  bytes.extend_from_slice(&0u16.to_le_bytes());
  bytes.extend_from_slice(&[17, 0, 0, 0]);

  bytes
}

/// `r1_decl_lmap`, the declaration xrLC writes for a lightmapped surface.
pub(crate) fn lightmapped_declaration() -> Vec<u8> {
  let mut bytes: Vec<u8> = element(0, 2, 0, 0);

  bytes.extend(element(12, 4, 3, 0));
  bytes.extend(element(16, 4, 6, 0));
  bytes.extend(element(20, 4, 7, 0));
  bytes.extend(element(24, 6, 5, 0));
  bytes.extend(element(28, 6, 5, 1));
  bytes.extend(terminator());

  bytes
}

/// The declaration that carries a position and nothing else, which is what a fast path stores.
pub(crate) fn positions_declaration() -> Vec<u8> {
  let mut bytes: Vec<u8> = element(0, 2, 0, 0);

  bytes.extend(terminator());

  bytes
}

/// One lightmapped vertex, whose every field decodes to something the test can recognise.
pub(crate) fn lightmapped_vertex(x: f32, y: f32, z: f32) -> Vec<u8> {
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
pub(crate) fn position_vertex(x: f32, y: f32, z: f32) -> Vec<u8> {
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
pub(crate) fn geometry(buffers: &[GeomBuffer], indices: &[u16]) -> Vec<u8> {
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

  let mut bytes: Vec<u8> = chunk(LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID, &vertex_chunk);

  bytes.extend(chunk(LevelGeomFile::INDEX_BUFFERS_CHUNK_ID, &index_chunk));

  bytes
}

/// Opens render geometry the same way the tool does, from bytes already in hand.
pub(crate) fn open_geometry(bytes: Vec<u8>) -> LevelGeomSource<xrf_chunk::InMemoryChunkDataSource> {
  LevelGeomSource::open_from_bytes::<XRayByteOrder>(bytes).expect("readable render geometry")
}

/// An OGF header of the given model type and shader entry.
fn header(model_type: u8, shader_id: u16) -> Vec<u8> {
  let mut bytes: Vec<u8> = vec![4, model_type];

  bytes.extend_from_slice(&shader_id.to_le_bytes());

  for value in [0.0f32, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0] {
    bytes.extend_from_slice(&value.to_le_bytes());
  }

  chunk(OgfHeaderChunk::CHUNK_ID, &bytes)
}

/// A visual that draws the given range of the level's shared buffers.
pub(crate) fn drawable(
  shader_id: u16,
  vertex_base: u32,
  vertex_count: u32,
  index_base: u32,
  index_count: u32,
) -> Vec<u8> {
  drawable_of_buffer(shader_id, 0, vertex_base, vertex_count, index_base, index_count)
}

/// The same, drawing out of a named vertex buffer rather than the first.
pub(crate) fn drawable_of_buffer(
  shader_id: u16,
  vertex_buffer: u32,
  vertex_base: u32,
  vertex_count: u32,
  index_base: u32,
  index_count: u32,
) -> Vec<u8> {
  let mut bytes: Vec<u8> = header(0, shader_id);
  let mut container: Vec<u8> = Vec::new();

  for value in [vertex_buffer, vertex_base, vertex_count, 0, index_base, index_count] {
    container.extend_from_slice(&value.to_le_bytes());
  }

  bytes.extend(chunk(OgfGeometryContainerChunk::CHUNK_ID, &container));

  bytes
}

/// A visual that draws nothing itself and links the visuals that do.
pub(crate) fn hierarchy(children: &[u32]) -> Vec<u8> {
  let mut bytes: Vec<u8> = header(1, 0);
  let mut links: Vec<u8> = (children.len() as u32).to_le_bytes().to_vec();

  for child in children {
    links.extend_from_slice(&child.to_le_bytes());
  }

  bytes.extend(chunk(OgfChildrenLinkChunk::CHUNK_ID, &links));

  bytes
}

/// A visuals run of the given visuals, each already framed as its own numbered chunk payload.
pub(crate) fn visuals(run: &[Vec<u8>]) -> LevelVisualsChunk {
  let mut body: Vec<u8> = Vec::new();

  for (index, visual) in run.iter().enumerate() {
    body.extend(chunk(index as u32, visual));
  }

  let level: Vec<u8> = chunk(LevelVisualsChunk::CHUNK_ID, &body);

  LevelFile::read_visuals_from_bytes::<XRayByteOrder>(level)
    .expect("readable visuals")
    .expect("a run of visuals")
}

/// A shader table of the given raw entries, as the level stores them.
pub(crate) fn shaders(entries: &[&str]) -> LevelShadersChunk {
  LevelShadersChunk {
    entries: entries
      .iter()
      .map(|entry| xrf_level::LevelShaderEntry::parse(entry))
      .collect(),
  }
}

/// A tree: the same range of the shared buffers, stood where its own transform puts it.
pub(crate) fn tree(shader_id: u16, vertex_base: u32, vertex_count: u32, index_count: u32, at: f32) -> Vec<u8> {
  let mut bytes: Vec<u8> = drawable_of_buffer(shader_id, 0, vertex_base, vertex_count, 0, index_count);
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

  // Scale and bias, five floats each, which packing does not read but the chunk carries.
  for value in [0.0f32; 10] {
    definition.extend_from_slice(&value.to_le_bytes());
  }

  bytes.extend(chunk(OgfTreeDefinitionChunk::CHUNK_ID, &definition));

  bytes
}
