use xrf_error::{XrfError, XrfResult};

use crate::geom::buffers::level_geom_vertex_buffer::LevelGeomVertexBuffer;
use crate::geom::buffers::level_geom_vertex_element::LevelGeomVertexElement;

/// One vertex buffer of a level's render geometry with its vertices' bytes, a stride of its declaration each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelGeomVertexData {
  /// The elements a vertex is made of, without the terminator that ends them in the file.
  pub declaration: Vec<LevelGeomVertexElement>,
  pub bytes: Vec<u8>,
}

impl LevelGeomVertexData {
  /// Bytes one vertex of the declaration takes.
  ///
  /// # Errors
  ///
  /// Returns an error for a declaration naming a type `D3DDECLTYPE` does not.
  pub fn get_stride(&self) -> XrfResult<u32> {
    LevelGeomVertexElement::get_vertex_size(&self.declaration, LevelGeomVertexBuffer::VERTEX_STREAM)
      .ok_or_else(|| XrfError::new_invalid_error("A level vertex declaration names a type D3DDECLTYPE does not"))
  }

  /// Vertices the buffer holds.
  ///
  /// # Errors
  ///
  /// Returns an error for a declaration of no known stride, or bytes that are not whole vertices of it.
  pub fn get_vertex_count(&self) -> XrfResult<u32> {
    let stride: u32 = self.get_stride()?;

    if stride == 0 || !self.bytes.len().is_multiple_of(stride as usize) {
      return Err(XrfError::new_invalid_error(format!(
        "A level vertex buffer of stride {stride} holds {} bytes, not whole vertices",
        self.bytes.len()
      )));
    }

    u32::try_from(self.bytes.len() / stride as usize)
      .map_err(|_| XrfError::new_invalid_error("A level vertex buffer holds more vertices than 32 bits count"))
  }
}
