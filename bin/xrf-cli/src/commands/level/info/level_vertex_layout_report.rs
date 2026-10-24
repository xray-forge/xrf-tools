use serde::Serialize;
use xrf_level::LevelGeomVertexBuffer;

/// One vertex declaration and how much of a level's geometry is stored in it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVertexLayoutReport {
  declaration: String,
  /// Bytes one vertex occupies, absent when the declaration names a type `D3DDECLTYPE` does not.
  stride: Option<u32>,
  buffers: usize,
  vertices: u64,
}

impl LevelVertexLayoutReport {
  pub fn new(declaration: String, stride: Option<u32>, buffers: usize, vertices: u64) -> Self {
    Self {
      declaration,
      stride,
      buffers,
      vertices,
    }
  }

  /// A declaration as one line, so two buffers sharing a layout collapse into one reported row.
  pub fn describe(buffer: &LevelGeomVertexBuffer) -> String {
    buffer
      .declaration
      .iter()
      .map(|element| {
        format!(
          "{}{}:{}",
          element.get_usage_name(),
          element.usage_index,
          element.get_kind_name()
        )
      })
      .collect::<Vec<_>>()
      .join(" | ")
  }
}
