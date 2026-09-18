use serde::{Deserialize, Serialize};

use crate::geom::level_geom_vertex_element::LevelGeomVertexElement;

/// One vertex buffer of a level's render geometry, read down to its shape rather than its vertices.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeomVertexBuffer {
  /// The elements a vertex is made of, without the terminator that ends them in the file.
  pub declaration: Vec<LevelGeomVertexElement>,
  pub vertex_count: u32,
  /// Where the vertices start, as an offset into the chunk that holds them.
  pub payload_offset: u64,
}

impl LevelGeomVertexBuffer {
  /// The stream the renderer takes a level vertex's stride from, `GetDeclVertexSize(dcl, 0)`.
  pub const VERTEX_STREAM: u16 = 0;

  /// Bytes one vertex occupies.
  pub fn get_vertex_size(&self) -> Option<u32> {
    let mut size: u16 = 0;

    for element in &self.declaration {
      if !element.is_fed_from(Self::VERTEX_STREAM) {
        continue;
      }

      size = size.max(element.get_kind_size()? + element.offset);
    }

    Some(size as u32)
  }

  /// Bytes of vertices the buffer carries, which is what a reader of its shape alone steps over.
  pub fn get_payload_size(&self) -> Option<u64> {
    Some(self.vertex_count as u64 * self.get_vertex_size()? as u64)
  }
}
