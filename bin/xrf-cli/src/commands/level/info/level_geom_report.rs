use std::collections::BTreeMap;

use serde::Serialize;
use xrf_level::LevelGeomFile;

use crate::commands::level::info::level_vertex_layout_report::LevelVertexLayoutReport;

/// One render geometry file, reported by the shape of its buffers.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeomReport {
  pub vertex_buffers: usize,
  pub index_buffers: usize,
  pub slide_windows: usize,
  pub vertices: u64,
  pub indices: u64,
  /// Distinct vertex declarations, each with what it is spelled as and how much of the level uses it.
  pub layouts: Vec<LevelVertexLayoutReport>,
}

impl LevelGeomReport {
  pub fn of(geometry: &LevelGeomFile) -> Self {
    let mut layouts: BTreeMap<(String, Option<u32>), (usize, u64)> = BTreeMap::new();

    for buffer in &geometry.vertex_buffers {
      let entry: &mut (usize, u64) = layouts
        .entry((LevelVertexLayoutReport::describe(buffer), buffer.get_vertex_size()))
        .or_default();

      entry.0 += 1;
      entry.1 += buffer.vertex_count as u64;
    }

    Self {
      index_buffers: geometry.index_buffers.len(),
      indices: geometry.index_buffers.iter().map(|it| it.index_count as u64).sum(),
      layouts: layouts
        .into_iter()
        .map(|((declaration, stride), (buffers, vertices))| LevelVertexLayoutReport {
          buffers,
          declaration,
          stride,
          vertices,
        })
        .collect(),
      slide_windows: geometry.slide_windows.len(),
      vertex_buffers: geometry.vertex_buffers.len(),
      vertices: geometry.vertex_buffers.iter().map(|it| it.vertex_count as u64).sum(),
    }
  }
}
