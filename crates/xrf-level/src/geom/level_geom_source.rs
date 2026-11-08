use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::geom::buffers::level_geom_index_buffer::LevelGeomIndexBuffer;
use crate::geom::buffers::level_geom_vertex_buffer::LevelGeomVertexBuffer;
use crate::geom::level_geom_file::LevelGeomFile;
use crate::geom::level_vertex_range::LevelVertexRange;
use crate::geom::vertex::level_vertex::LevelVertex;
use crate::geom::vertex::level_vertex_bytes::LevelVertexBytes;
use crate::geom::vertex::level_vertex_layout::LevelVertexLayout;
use crate::geom::vertex::level_vertex_payload::LevelVertexPayload;
use crate::level::visuals::level_visual::LevelVisual;

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
      indices: ChunkReader::find_required_by_id(&chunks, LevelGeomFile::INDEX_BUFFERS_CHUNK_ID)?,
      vertices: ChunkReader::find_required_by_id(&chunks, LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID)?,
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
    let range: LevelVertexRange = self.locate_vertices(buffer, base, count)?;

    Self::read_payload_with(&self.vertices, range.offset, range.size, "vertices", |bytes| {
      LevelVertexBytes::each(&range.layout, bytes)
        .map(|vertex| vertex.decode::<T>())
        .collect()
    })
  }

  /// Reads one visual's vertices as they are stored, with the declaration that says where each attribute sits.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer does not exist, the range reaches past its vertices, or its declaration is
  /// one xrLC does not write.
  pub fn read_vertex_payload(&self, buffer: u32, base: u32, count: u32) -> XrfResult<LevelVertexPayload> {
    let range: LevelVertexRange = self.locate_vertices(buffer, base, count)?;
    let bytes: Vec<u8> = Self::read_payload_with(&self.vertices, range.offset, range.size, "vertices", <[u8]>::to_vec)?;

    Ok(LevelVertexPayload::new(range.layout, bytes))
  }

  /// Where one visual's vertices sit in their buffer, and the declaration they were stored in.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer does not exist, the range reaches past its vertices, or its declaration is
  /// one xrLC does not write.
  fn locate_vertices(&self, buffer: u32, base: u32, count: u32) -> XrfResult<LevelVertexRange> {
    let declared: &LevelGeomVertexBuffer = self
      .file
      .vertex_buffers
      .get(buffer as usize)
      .ok_or_else(|| Self::unknown_buffer("vertex", buffer, self.file.vertex_buffers.len()))?;

    Self::require_range("vertex", buffer, base, count, declared.vertex_count)?;

    let layout: LevelVertexLayout = LevelVertexLayout::of(declared)?;

    Ok(LevelVertexRange {
      offset: declared.payload_offset + u64::from(base) * u64::from(layout.get_stride()),
      size: count as usize * layout.get_stride() as usize,
      layout,
    })
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

    Self::read_payload_with(
      &self.indices,
      declared.payload_offset + u64::from(base) * LevelGeomIndexBuffer::INDEX_SIZE,
      count as usize * LevelGeomIndexBuffer::INDEX_SIZE as usize,
      "indices",
      |bytes| {
        bytes
          .as_chunks::<2>()
          .0
          .iter()
          .map(|index| T::read_u16(index))
          .collect()
      },
    )
  }

  /// Reads the indices `visual` draws at full detail: its whole index range, cut to its first slide window where it is
  /// progressive (see [`LevelVisual::get_full_detail_window`]); `None` for a visual drawing nothing from the buffers.
  ///
  /// # Errors
  ///
  /// Returns an error where the indices or the window cannot be read, or the window reaches past the visual's indices.
  pub fn read_full_detail_indices<T: ByteOrder>(&self, visual: &LevelVisual) -> XrfResult<Option<Vec<u16>>> {
    let Some(container) = &visual.geometry else {
      return Ok(None);
    };
    let indices: Vec<u16> =
      self.read_indices::<T>(container.index_buffer_id, container.index_base, container.index_count)?;
    let Some(window) = visual.get_full_detail_window(&self.file)? else {
      return Ok(Some(indices));
    };
    let start: usize = window.offset as usize;
    let end: usize = start + usize::from(window.triangles) * 3;

    indices.get(start..end).map(|run| Some(run.to_vec())).ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "A visual's full-detail window [{start}..{end}) reaches past its {} indices",
        indices.len()
      ))
    })
  }

  /// Hands the bytes of one payload that a range covers to `visit`, leaving the chunk where it was.
  fn read_payload_with<R>(
    reader: &ChunkReader<D>,
    offset: u64,
    size: usize,
    what: &str,
    visit: impl FnOnce(&[u8]) -> R,
  ) -> XrfResult<R> {
    reader.data.read_at_with(offset, size, visit).map_err(|error| {
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
