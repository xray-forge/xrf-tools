use std::collections::BTreeMap;

use serde::Serialize;
use xrf_level::LevelGeomFile;

use crate::commands::level::info::level_vertex_layout_report::LevelVertexLayoutReport;

/// One render geometry file, reported by the shape of its buffers.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelGeomReport {
  index_buffers: usize,
  slide_windows: usize,
  indices: u64,
  /// Distinct vertex declarations, each with what it is spelled as and how much of the level uses it; their buffers
  /// and vertices add up to the file's.
  layouts: Vec<LevelVertexLayoutReport>,
}

impl LevelGeomReport {
  pub(crate) fn of(geometry: &LevelGeomFile) -> Self {
    let mut layouts: BTreeMap<(String, Option<u32>), (usize, u64)> = BTreeMap::new();

    for buffer in &geometry.vertex_buffers {
      let entry: &mut (usize, u64) = layouts
        .entry((LevelVertexLayoutReport::describe(buffer), buffer.get_vertex_size()))
        .or_default();

      entry.0 += 1;
      entry.1 += u64::from(buffer.vertex_count);
    }

    Self {
      index_buffers: geometry.index_buffers.len(),
      indices: geometry.index_buffers.iter().map(|it| u64::from(it.index_count)).sum(),
      layouts: layouts
        .into_iter()
        .map(|((declaration, stride), (buffers, vertices))| {
          LevelVertexLayoutReport::new(declaration, stride, buffers, vertices)
        })
        .collect(),
      slide_windows: geometry.slide_windows.len(),
    }
  }
}
