use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::geom::buffers::level_geom_index_buffer::LevelGeomIndexBuffer;
use crate::geom::buffers::level_geom_vertex_buffer::LevelGeomVertexBuffer;
use crate::geom::level_geom_file::LevelGeomFile;
use crate::geom::vertex::level_vertex::LevelVertex;
use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;
use crate::geom::vertex::level_vertex_payload::LevelVertexPayload;

/// A level's render geometry with its payloads still where they were, able to serve any range a visual names.
pub struct LevelGeomSource<D: ChunkDataSource> {
  file: LevelGeomFile,
  vertices: ChunkReader<D>,
  indices: ChunkReader<D>,
}

impl LevelGeomSource<xrf_chunk::InMemoryChunkDataSource> {
  /// Opens render geometry from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened or is not render geometry this reads.
  pub fn open_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::open_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level render geometry was not opened: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Opens render geometry from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or is not render geometry this reads.
  pub fn open_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::open_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Opens render geometry from bytes already in hand, which is how an archived level arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not render geometry this reads.
  pub fn open_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::open_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }
}

impl<D: ChunkDataSource> LevelGeomSource<D> {
  /// Opens render geometry from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a required chunk is absent or the geometry's shape cannot be read.
  pub fn open_from_chunk<T: ByteOrder, S: ChunkDataSource>(
    reader: &mut ChunkReader<S>,
  ) -> XrfResult<LevelGeomSource<S>> {
    let chunks: Vec<ChunkReader<S>> = reader.read_children()?;

    Ok(LevelGeomSource {
      file: LevelGeomFile::read_from_children::<T, _>(&chunks)?,
      indices: find_required_chunk_by_id(&chunks, LevelGeomFile::INDEX_BUFFERS_CHUNK_ID)?,
      vertices: find_required_chunk_by_id(&chunks, LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID)?,
    })
  }

  /// The geometry's shape: what buffers there are, and what a vertex of each is made of.
  pub const fn get_file(&self) -> &LevelGeomFile {
    &self.file
  }

  /// Reads one visual's vertices, decoded out of whichever declaration stored them.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer does not exist, the range reaches past its vertices, or its declaration is
  /// one xrLC does not write.
  pub fn read_vertices<T: ByteOrder>(&self, buffer: u32, base: u32, count: u32) -> XrfResult<Vec<LevelVertex>> {
    let payload: LevelVertexPayload = self.read_vertex_payload(buffer, base, count)?;

    Ok(payload.vertices().map(|vertex| vertex.decode::<T>()).collect())
  }

  /// Reads one visual's vertices as they are stored, with the declaration that says where each attribute sits.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer does not exist, the range reaches past its vertices, or its declaration is
  /// one xrLC does not write.
  pub fn read_vertex_payload(&self, buffer: u32, base: u32, count: u32) -> XrfResult<LevelVertexPayload> {
    let declared: &LevelGeomVertexBuffer = self
      .file
      .vertex_buffers
      .get(buffer as usize)
      .ok_or_else(|| Self::unknown_buffer("vertex", buffer, self.file.vertex_buffers.len()))?;

    Self::require_range("vertex", buffer, base, count, declared.vertex_count)?;

    let layout: LevelVertexLayout = LevelVertexLayout::of(declared)?;

    let bytes: Vec<u8> = Self::read_payload(
      &self.vertices,
      declared.payload_offset + u64::from(base) * u64::from(layout.get_stride()),
      count as usize * layout.get_stride() as usize,
      "vertices",
    )?;

    Ok(LevelVertexPayload::new(layout, bytes))
  }

  /// Reads one visual's indices.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer does not exist or the range reaches past its indices.
  pub fn read_indices<T: ByteOrder>(&self, buffer: u32, base: u32, count: u32) -> XrfResult<Vec<u16>> {
    let declared: &LevelGeomIndexBuffer = self
      .file
      .index_buffers
      .get(buffer as usize)
      .ok_or_else(|| Self::unknown_buffer("index", buffer, self.file.index_buffers.len()))?;

    Self::require_range("index", buffer, base, count, declared.index_count)?;

    let payload: Vec<u8> = Self::read_payload(
      &self.indices,
      declared.payload_offset + u64::from(base) * LevelGeomIndexBuffer::INDEX_SIZE,
      count as usize * LevelGeomIndexBuffer::INDEX_SIZE as usize,
      "indices",
    )?;

    Ok(
      payload
        .as_chunks::<2>()
        .0
        .iter()
        .map(|index| T::read_u16(index))
        .collect(),
    )
  }

  /// Takes the bytes of one payload that a range covers, leaving the chunk where it was.
  fn read_payload(reader: &ChunkReader<D>, offset: u64, size: usize, what: &str) -> XrfResult<Vec<u8>> {
    reader.data.read_at(offset, size).map_err(|error| {
      XrfError::new_read_error(format!(
        "Level render geometry {what} were not read at offset {offset}: {error}"
      ))
    })
  }

  /// Refuses a range that reaches past what the buffer declared, rather than reading a neighbour's bytes.
  fn require_range(kind: &str, buffer: u32, base: u32, count: u32, declared: u32) -> XrfResult {
    match base.checked_add(count) {
      Some(end) if end <= declared => Ok(()),
      _ => Err(XrfError::new_invalid_error(format!(
        "Unexpected level render geometry range [{base}..{base}+{count}) in {kind} buffer {buffer}, \
         which declares {declared}"
      ))),
    }
  }

  /// Refuses a buffer id no buffer answers to.
  fn unknown_buffer(kind: &str, buffer: u32, count: usize) -> XrfError {
    XrfError::new_invalid_error(format!(
      "Unexpected level render geometry {kind} buffer {buffer}, of {count} the geometry carries"
    ))
  }
}
