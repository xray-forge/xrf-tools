use serde::{Deserialize, Serialize};

/// One index buffer of a level's render geometry, read down to its size rather than its indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeomIndexBuffer {
  pub index_count: u32,
}

impl LevelGeomIndexBuffer {
  /// Bytes one index occupies; the engine creates the buffer as `iCount * 2` and reads that many.
  pub const INDEX_SIZE: u64 = 2;

  /// Bytes of indices the buffer carries, which is what a reader of its shape alone steps over.
  pub const fn get_payload_size(&self) -> u64 {
    self.index_count as u64 * Self::INDEX_SIZE
  }

  /// Triangles the indices draw, taking them as a triangle list.
  pub const fn get_triangles_count(&self) -> u32 {
    self.index_count / 3
  }
}
