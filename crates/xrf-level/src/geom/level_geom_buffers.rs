use std::path::Path;

use xrf_chunk::{ChunkReader, ChunkWriter, XRayWrite};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, narrow_format_size};

use crate::geom::buffers::level_geom_vertex_element::LevelGeomVertexElement;
use crate::geom::level_geom_file::LevelGeomFile;
use crate::geom::level_geom_source::LevelGeomSource;
use crate::geom::level_geom_vertex_data::LevelGeomVertexData;
use crate::geom::window::level_geom_slide_window_item::LevelGeomSlideWindowItem;
use crate::level::level_header_chunk::LevelHeaderChunk;

/// A level's render geometry with every buffer's bytes, as xrLC writes `level.geom` or `level.geomx` (`SaveGEOMs`);
/// [`LevelGeomFile`] reads only its shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelGeomBuffers {
  /// `hdrLEVEL`, of which xrLC sets the version and leaves the quality as it finds it.
  pub header: LevelHeaderChunk,
  pub vertex_buffers: Vec<LevelGeomVertexData>,
  pub index_buffers: Vec<Vec<u16>>,
  pub slide_windows: Vec<LevelGeomSlideWindowItem>,
}

impl LevelGeomBuffers {
  /// Reads render geometry whole from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not render geometry this reads.
  pub fn read_from_path<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_bytes(std::fs::read(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level render geometry was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads render geometry whole from bytes in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the header is absent, or the bytes are not render geometry this reads.
  pub fn read_from_bytes(bytes: Vec<u8>) -> XrfResult<Self> {
    let chunks = ChunkReader::from_vec(bytes)?.read_children()?;
    let header: LevelHeaderChunk = ChunkReader::find_required_by_id(&chunks, LevelHeaderChunk::CHUNK_ID)?.read_xr()?;
    let source = LevelGeomSource::open_from_children(&chunks)?;
    let file: &LevelGeomFile = source.get_file();
    let mut vertex_buffers: Vec<LevelGeomVertexData> = Vec::with_capacity(file.vertex_buffers.len());
    let mut index_buffers: Vec<Vec<u16>> = Vec::with_capacity(file.index_buffers.len());

    for (index, buffer) in file.vertex_buffers.iter().enumerate() {
      vertex_buffers.push(LevelGeomVertexData {
        declaration: buffer.declaration.clone(),
        bytes: source
          .read_vertex_payload(index as u32, 0, buffer.vertex_count)?
          .get_bytes()
          .to_vec(),
      });
    }

    for (index, buffer) in file.index_buffers.iter().enumerate() {
      index_buffers.push(source.read_indices(index as u32, 0, buffer.index_count)?);
    }

    Ok(Self {
      header,
      vertex_buffers,
      index_buffers,
      slide_windows: file.slide_windows.clone(),
    })
  }

  /// Writes the file as xrLC does: the header, the vertex buffers, the index buffers, the slide windows.
  ///
  /// # Errors
  ///
  /// Returns an error when a vertex buffer's bytes are not whole vertices, a count overflows 32 bits, or the writer
  /// fails.
  pub fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut header: ChunkWriter = ChunkWriter::new();

    header.write_xr(&self.header)?;
    header.flush_chunk_into(&mut writer.buffer, LevelHeaderChunk::CHUNK_ID)?;

    let mut vertices: ChunkWriter = ChunkWriter::new();

    vertices.write_u32(narrow_format_size(
      self.vertex_buffers.len(),
      "level geometry vertex buffers",
    )?)?;

    for buffer in &self.vertex_buffers {
      for element in buffer.declaration.iter().chain([&LevelGeomVertexElement::END]) {
        vertices.write_xr(element)?;
      }

      vertices.write_u32(buffer.get_vertex_count()?)?;
      vertices.buffer.extend_from_slice(&buffer.bytes);
    }

    vertices.flush_chunk_into(&mut writer.buffer, LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID)?;

    let mut indices: ChunkWriter = ChunkWriter::new();

    indices.write_u32(narrow_format_size(
      self.index_buffers.len(),
      "level geometry index buffers",
    )?)?;

    for buffer in &self.index_buffers {
      indices.write_u32(narrow_format_size(buffer.len(), "level geometry indices")?)?;

      for index in buffer {
        indices.write_u16(*index)?;
      }
    }

    indices.flush_chunk_into(&mut writer.buffer, LevelGeomFile::INDEX_BUFFERS_CHUNK_ID)?;

    let mut windows: ChunkWriter = ChunkWriter::new();

    windows.write_u32(narrow_format_size(
      self.slide_windows.len(),
      "level geometry slide windows",
    )?)?;

    for item in &self.slide_windows {
      for reserved in item.reserved {
        windows.write_u32(reserved)?;
      }

      windows.write_u32(narrow_format_size(item.windows.len(), "level geometry detail levels")?)?;

      for window in &item.windows {
        windows.write_xr(window)?;
      }
    }

    windows.flush_chunk_into(&mut writer.buffer, LevelGeomFile::SLIDE_WINDOWS_CHUNK_ID)?;

    Ok(())
  }
}
