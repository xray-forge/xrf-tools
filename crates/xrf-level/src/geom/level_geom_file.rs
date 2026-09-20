use std::fs::File;
use std::io::SeekFrom;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, find_optional_chunk_by_id, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::geom::buffers::level_geom_index_buffer::LevelGeomIndexBuffer;
use crate::geom::buffers::level_geom_vertex_buffer::LevelGeomVertexBuffer;
use crate::geom::buffers::level_geom_vertex_element::LevelGeomVertexElement;
use crate::geom::window::level_geom_slide_window::LevelGeomSlideWindow;
use crate::geom::window::level_geom_slide_window_item::LevelGeomSlideWindowItem;

/// The render geometry of a level, `level.geom` and its detail twin `level.geomX`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeomFile {
  pub vertex_buffers: Vec<LevelGeomVertexBuffer>,
  pub index_buffers: Vec<LevelGeomIndexBuffer>,
  /// The progressive meshes, absent from `level.geomX`, which the engine reads buffers from and nothing else.
  pub slide_windows: Vec<LevelGeomSlideWindowItem>,
}

impl LevelGeomFile {
  /// `fsL_VB` (`Common/LevelStructure.hpp`).
  pub const VERTEX_BUFFERS_CHUNK_ID: u32 = 9;

  /// `fsL_IB`.
  pub const INDEX_BUFFERS_CHUNK_ID: u32 = 10;

  /// `fsL_SWIS`, which `CRender::LoadSWIs` looks for and tolerates the absence of.
  pub const SLIDE_WINDOWS_CHUNK_ID: u32 = 11;

  /// Reads render geometry from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not render geometry this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level render geometry was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads render geometry from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not render geometry this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how archived geometry arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not render geometry this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a required chunk is absent, a declaration names a type `D3DDECLTYPE` does not, or a
  /// chunk does not end exactly where its buffers say it should.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Self::read_from_children::<T, _>(&reader.read_children()?)
  }

  /// Reads render geometry from children already read from it.
  ///
  /// # Errors
  ///
  /// Returns an error when a required chunk is absent, a declaration names a type `D3DDECLTYPE` does not, or a
  /// chunk does not end exactly where its buffers say it should.
  pub fn read_from_children<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    let mut vertices: ChunkReader<D> = find_required_chunk_by_id(chunks, Self::VERTEX_BUFFERS_CHUNK_ID)?;
    let vertex_buffers: Vec<LevelGeomVertexBuffer> = Self::read_vertex_buffers::<T, D>(&mut vertices)?;

    vertices.assert_read("Expect all data to be read from level render geometry vertex buffers chunk")?;

    let mut indices: ChunkReader<D> = find_required_chunk_by_id(chunks, Self::INDEX_BUFFERS_CHUNK_ID)?;
    let index_buffers: Vec<LevelGeomIndexBuffer> = Self::read_index_buffers::<T, D>(&mut indices)?;

    indices.assert_read("Expect all data to be read from level render geometry index buffers chunk")?;

    let slide_windows: Vec<LevelGeomSlideWindowItem> =
      match find_optional_chunk_by_id(chunks, Self::SLIDE_WINDOWS_CHUNK_ID) {
        Some(mut chunk) => {
          let windows: Vec<LevelGeomSlideWindowItem> = Self::read_slide_windows::<T, D>(&mut chunk)?;

          chunk.assert_read("Expect all data to be read from level render geometry slide windows chunk")?;

          windows
        }
        None => Vec::new(),
      };

    Ok(Self {
      vertex_buffers,
      index_buffers,
      slide_windows,
    })
  }

  /// Reads every vertex buffer's declaration and count, stepping over its vertices.
  fn read_vertex_buffers<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<LevelGeomVertexBuffer>> {
    let count: u32 = reader.read_u32::<T>()?;
    let mut buffers: Vec<LevelGeomVertexBuffer> = reader.new_bounded_vec(
      count as u64,
      LevelGeomVertexElement::SERIALIZED_SIZE + 4,
      "level render geometry vertex buffers",
    )?;

    for _ in 0..count {
      let declaration: Vec<LevelGeomVertexElement> = Self::read_declaration::<T, D>(reader)?;
      let vertex_count: u32 = reader.read_u32::<T>()?;

      let buffer: LevelGeomVertexBuffer = LevelGeomVertexBuffer {
        declaration,
        payload_offset: reader.read_bytes_len(),
        vertex_count,
      };

      let payload: u64 = buffer.get_payload_size().ok_or_else(|| {
        XrfError::new_invalid_error(
          "Unexpected level render geometry vertex declaration naming a type D3DDECLTYPE does not",
        )
      })?;

      Self::skip::<D>(reader, payload, "vertices")?;

      buffers.push(buffer);
    }

    Ok(buffers)
  }

  /// Reads one declaration up to and including its terminator, keeping everything before it.
  fn read_declaration<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<LevelGeomVertexElement>> {
    let mut declaration: Vec<LevelGeomVertexElement> = Vec::new();

    loop {
      let element: LevelGeomVertexElement = LevelGeomVertexElement::read::<T, D>(reader)?;

      if element.is_terminator() {
        return Ok(declaration);
      }

      if declaration.len() >= LevelGeomVertexElement::MAXIMUM_LENGTH {
        return Err(XrfError::new_invalid_error(format!(
          "Unexpected level render geometry vertex declaration of more than {} elements",
          LevelGeomVertexElement::MAXIMUM_LENGTH
        )));
      }

      declaration.push(element);
    }
  }

  /// Reads every index buffer's count, stepping over its indices.
  fn read_index_buffers<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<LevelGeomIndexBuffer>> {
    let count: u32 = reader.read_u32::<T>()?;
    let mut buffers: Vec<LevelGeomIndexBuffer> =
      reader.new_bounded_vec(count as u64, 4, "level render geometry index buffers")?;

    for _ in 0..count {
      let index_count: u32 = reader.read_u32::<T>()?;

      let buffer: LevelGeomIndexBuffer = LevelGeomIndexBuffer {
        index_count,
        payload_offset: reader.read_bytes_len(),
      };

      Self::skip::<D>(reader, buffer.get_payload_size(), "indices")?;

      buffers.push(buffer);
    }

    Ok(buffers)
  }

  /// Reads every progressive mesh's chain of detail levels, which are small enough to keep.
  fn read_slide_windows<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<LevelGeomSlideWindowItem>> {
    let count: u32 = reader.read_u32::<T>()?;
    let mut items: Vec<LevelGeomSlideWindowItem> = reader.new_bounded_vec(
      count as u64,
      LevelGeomSlideWindowItem::HEADER_SIZE,
      "level render geometry slide windows",
    )?;

    for _ in 0..count {
      let reserved: [u32; 4] = [
        reader.read_u32::<T>()?,
        reader.read_u32::<T>()?,
        reader.read_u32::<T>()?,
        reader.read_u32::<T>()?,
      ];

      let count: u32 = reader.read_u32::<T>()?;
      let mut windows: Vec<LevelGeomSlideWindow> = reader.new_bounded_vec(
        count as u64,
        LevelGeomSlideWindow::SERIALIZED_SIZE,
        "level render geometry detail levels",
      )?;

      for _ in 0..count {
        windows.push(reader.read_xr::<T, _>()?);
      }

      items.push(LevelGeomSlideWindowItem { reserved, windows });
    }

    Ok(items)
  }

  /// Steps over a buffer's payload, refusing one that reaches past the chunk holding it.
  fn skip<D: ChunkDataSource>(reader: &mut ChunkReader<D>, size: u64, what: &str) -> XrfResult {
    if size > reader.read_bytes_remain() {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level render geometry buffer claiming {size} bytes of {what}, past the end of its chunk"
      )));
    }

    reader.data.set_seek(SeekFrom::Start(reader.read_bytes_len() + size))?;

    Ok(())
  }
}

impl LevelGeomFile {
  /// Vertices across every buffer, which is what the level costs to hold.
  pub fn get_vertices_count(&self) -> u64 {
    self
      .vertex_buffers
      .iter()
      .map(|buffer| buffer.vertex_count as u64)
      .sum()
  }

  /// Indices across every buffer.
  pub fn get_indices_count(&self) -> u64 {
    self.index_buffers.iter().map(|buffer| buffer.index_count as u64).sum()
  }

  /// Triangles the indices draw, taking them as triangle lists.
  pub fn get_triangles_count(&self) -> u64 {
    self
      .index_buffers
      .iter()
      .map(|buffer| buffer.get_triangles_count() as u64)
      .sum()
  }

  /// Detail levels across every progressive mesh.
  pub fn get_detail_levels_count(&self) -> usize {
    self.slide_windows.iter().map(|item| item.windows.len()).sum()
  }
}
