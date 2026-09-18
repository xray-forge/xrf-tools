use std::fs::File;
use std::io::{Read, SeekFrom};
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::format_path;

use crate::geom::level_geom_file::LevelGeomFile;
use crate::geom::level_geom_index_buffer::LevelGeomIndexBuffer;
use crate::geom::level_geom_vertex_buffer::LevelGeomVertexBuffer;
use crate::geom::level_vertex::LevelVertex;
use crate::geom::level_vertex_layout::LevelVertexLayout;

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
  pub fn read_vertices<T: ByteOrder>(&mut self, buffer: u32, base: u32, count: u32) -> XrfResult<Vec<LevelVertex>> {
    let declared: &LevelGeomVertexBuffer = self
      .file
      .vertex_buffers
      .get(buffer as usize)
      .ok_or_else(|| Self::unknown_buffer("vertex", buffer, self.file.vertex_buffers.len()))?;

    Self::require_range("vertex", buffer, base, count, declared.vertex_count)?;

    let layout: LevelVertexLayout = LevelVertexLayout::of(declared)?;
    let stride: usize = layout.stride as usize;
    let payload: Vec<u8> = Self::read_payload(
      &mut self.vertices,
      declared.payload_offset + u64::from(base) * u64::from(layout.stride),
      count as usize * stride,
      "vertices",
    )?;

    Ok(
      payload
        .chunks_exact(stride)
        .map(|vertex| Self::decode_vertex::<T>(&layout, vertex))
        .collect(),
    )
  }

  /// Reads one visual's indices.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffer does not exist or the range reaches past its indices.
  pub fn read_indices<T: ByteOrder>(&mut self, buffer: u32, base: u32, count: u32) -> XrfResult<Vec<u16>> {
    let declared: &LevelGeomIndexBuffer = self
      .file
      .index_buffers
      .get(buffer as usize)
      .ok_or_else(|| Self::unknown_buffer("index", buffer, self.file.index_buffers.len()))?;

    Self::require_range("index", buffer, base, count, declared.index_count)?;

    let payload: Vec<u8> = Self::read_payload(
      &mut self.indices,
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

  /// Turns one vertex's bytes into what xrLC had before it quantized them.
  fn decode_vertex<T: ByteOrder>(layout: &LevelVertexLayout, vertex: &[u8]) -> LevelVertex {
    let (normal, hemi): (Vector3d, u8) = match layout.get_normal_offset() {
      Some(offset) => LevelVertex::decode_direction(Self::take_four(vertex, offset)),
      None => (Vector3d { x: 0.0, y: 0.0, z: 0.0 }, 0),
    };

    // The low byte of each base coordinate rides in a tangent or binormal alpha, so the coordinate is rebuilt from
    // two elements rather than one. A tree carries neither, and zero is exact there.
    let fraction_u: u8 = layout
      .get_tangent_offset()
      .map_or(0, |offset| Self::take_four(vertex, offset)[3]);
    let fraction_v: u8 = layout
      .get_binormal_offset()
      .map_or(0, |offset| Self::take_four(vertex, offset)[3]);

    LevelVertex {
      color: layout
        .get_color_offset()
        .map(|offset| LevelVertex::decode_color(Self::take_four(vertex, offset))),
      hemi,
      lightmap_coordinate: layout.get_lightmap_coordinate_offset().map(|offset| {
        (
          f32::from(Self::take_short::<T>(vertex, offset)) / LevelVertexLayout::LIGHTMAP_QUANT,
          f32::from(Self::take_short::<T>(vertex, offset + 2)) / LevelVertexLayout::LIGHTMAP_QUANT,
        )
      }),
      normal,
      position: Self::take_position::<T>(vertex, layout.get_position_offset()),
      texture_coordinate: layout.get_texture_coordinate_offset().map_or((0.0, 0.0), |offset| {
        (
          layout.rebuild_coordinate(Self::take_short::<T>(vertex, offset), fraction_u),
          layout.rebuild_coordinate(Self::take_short::<T>(vertex, offset + 2), fraction_v),
        )
      }),
    }
  }

  /// The four bytes of a `D3DCOLOR` element. The layout validated the offset, so the slice is there.
  fn take_four(vertex: &[u8], offset: u16) -> [u8; 4] {
    let at: usize = offset as usize;
    let mut bytes: [u8; 4] = [0; 4];

    bytes.copy_from_slice(&vertex[at..at + 4]);

    bytes
  }

  /// One signed 16-bit component of a coordinate element.
  fn take_short<T: ByteOrder>(vertex: &[u8], offset: u16) -> i16 {
    let at: usize = offset as usize;

    T::read_i16(&vertex[at..at + 2])
  }

  /// The three floats of a position element.
  fn take_position<T: ByteOrder>(vertex: &[u8], offset: u16) -> Vector3d {
    let at: usize = offset as usize;

    Vector3d {
      x: T::read_f32(&vertex[at..at + 4]),
      y: T::read_f32(&vertex[at + 4..at + 8]),
      z: T::read_f32(&vertex[at + 8..at + 12]),
    }
  }

  /// Seeks a chunk to one payload and takes the bytes of it that a range covers.
  fn read_payload(reader: &mut ChunkReader<D>, offset: u64, size: usize, what: &str) -> XrfResult<Vec<u8>> {
    reader.data.set_seek(SeekFrom::Start(offset))?;

    let mut payload: Vec<u8> = vec![0; size];

    reader.read_exact(&mut payload).map_err(|error| {
      XrfError::new_read_error(format!(
        "Level render geometry {what} were not read at offset {offset}: {error}"
      ))
    })?;

    Ok(payload)
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
