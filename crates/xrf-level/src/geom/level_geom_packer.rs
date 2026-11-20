use xrf_error::{XrfError, XrfResult};
use xrf_utils::narrow_format_size;

use crate::geom::buffers::level_geom_vertex_element::LevelGeomVertexElement;
use crate::geom::level_geom_buffers::LevelGeomBuffers;
use crate::geom::level_geom_range::LevelGeomRange;
use crate::geom::level_geom_vertex_data::LevelGeomVertexData;
use crate::geom::window::level_geom_slide_window_item::LevelGeomSlideWindowItem;
use crate::level::level_header_chunk::LevelHeaderChunk;

/// Visuals' vertices and indices gathered into a level's shared buffers as xrLC gathers them (`VBContainer` and
/// `IBContainer`, `xrLC/vbm.h`).
///
/// A visual's vertices go to the first buffer of the same declaration they fit in whole, under
/// [`Self::MAXIMUM_VERTEX_BYTES`] and [`Self::MAXIMUM_VERTICES`], or start a buffer; its indices to the first index
/// buffer that stays under [`Self::INDEX_LIMIT`] with them, or start one.
#[derive(Clone, Debug, Default)]
pub struct LevelGeomPacker {
  vertex_buffers: Vec<LevelGeomVertexData>,
  index_buffers: Vec<Vec<u16>>,
}

impl LevelGeomPacker {
  /// `c_VB_maxSize`, the bytes a vertex buffer holds at most.
  pub const MAXIMUM_VERTEX_BYTES: usize = 4096 * 1024;

  /// `c_VB_maxVertices`, the vertices a buffer holds at most, which 16-bit indices address.
  pub const MAXIMUM_VERTICES: usize = 65535;

  /// `IBContainer::LIMIT`: an index buffer holds fewer indices than this.
  pub const INDEX_LIMIT: usize = 1024 * 1024;

  /// Packs a visual's vertices, `bytes` whole vertices of `declaration`, and says where they went.
  ///
  /// # Errors
  ///
  /// Returns an error for no vertex, a declaration of no known stride, bytes that are not whole vertices of it, or
  /// buffers past what the file numbers.
  pub fn add_vertices(&mut self, declaration: &[LevelGeomVertexElement], bytes: &[u8]) -> XrfResult<LevelGeomRange> {
    let added: LevelGeomVertexData = LevelGeomVertexData {
      declaration: declaration.to_vec(),
      bytes: bytes.to_vec(),
    };
    let stride: usize = added.get_stride()? as usize;
    let count: usize = added.get_vertex_count()? as usize;

    if count == 0 {
      return Err(XrfError::new_invalid_error("A visual of no vertex has nothing to pack"));
    }

    for (buffer, held) in self.vertex_buffers.iter_mut().enumerate() {
      let already: usize = held.bytes.len() / stride;

      if held.declaration != declaration
        || held.bytes.len() + bytes.len() > Self::MAXIMUM_VERTEX_BYTES
        || already + count > Self::MAXIMUM_VERTICES
      {
        continue;
      }

      held.bytes.extend_from_slice(bytes);

      return Ok(LevelGeomRange {
        buffer: narrow_format_size(buffer, "A vertex buffer number")?,
        start: narrow_format_size(already, "A vertex buffer's start")?,
      });
    }

    self.vertex_buffers.push(added);

    Ok(LevelGeomRange {
      buffer: narrow_format_size(self.vertex_buffers.len() - 1, "A vertex buffer number")?,
      start: 0,
    })
  }

  /// Packs a visual's indices and says where they went.
  ///
  /// # Errors
  ///
  /// Returns an error when the buffers are past what the file numbers.
  pub fn add_indices(&mut self, indices: &[u16]) -> XrfResult<LevelGeomRange> {
    for (buffer, held) in self.index_buffers.iter_mut().enumerate() {
      if held.len() + indices.len() < Self::INDEX_LIMIT {
        let start: usize = held.len();

        held.extend_from_slice(indices);

        return Ok(LevelGeomRange {
          buffer: narrow_format_size(buffer, "An index buffer number")?,
          start: narrow_format_size(start, "An index buffer's start")?,
        });
      }
    }

    self.index_buffers.push(indices.to_vec());

    Ok(LevelGeomRange {
      buffer: narrow_format_size(self.index_buffers.len() - 1, "An index buffer number")?,
      start: 0,
    })
  }

  /// The geometry file of everything packed, under `header`, with `slide_windows`.
  pub fn into_buffers(
    self,
    header: LevelHeaderChunk,
    slide_windows: Vec<LevelGeomSlideWindowItem>,
  ) -> LevelGeomBuffers {
    LevelGeomBuffers {
      header,
      vertex_buffers: self.vertex_buffers,
      index_buffers: self.index_buffers,
      slide_windows,
    }
  }
}
