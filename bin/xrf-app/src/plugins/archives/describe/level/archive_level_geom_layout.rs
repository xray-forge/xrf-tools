use std::collections::BTreeMap;

use serde::Serialize;
use xrf_level::LevelGeomVertexBuffer;

/// One vertex layout a level's geometry is built from, and how much is built with it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelGeomLayout {
  /// Bytes one vertex of this layout occupies.
  pub stride: u32,
  /// Elements the declaration names, which is what the stride is made of.
  pub elements: usize,
  pub buffers: usize,
  pub vertices: u64,
}

impl ArchiveLevelGeomLayout {
  /// Every layout the buffers use, grouped, widest first.
  pub fn of_all(buffers: &[LevelGeomVertexBuffer]) -> Vec<Self> {
    let mut grouped: BTreeMap<(u32, usize), (usize, u64)> = BTreeMap::new();

    for buffer in buffers {
      let stride: u32 = buffer.get_vertex_size().unwrap_or_default();
      let entry: &mut (usize, u64) = grouped.entry((stride, buffer.declaration.len())).or_default();

      entry.0 += 1;
      entry.1 += u64::from(buffer.vertex_count);
    }

    let mut layouts: Vec<Self> = grouped
      .into_iter()
      .map(|((stride, elements), (buffers, vertices))| Self {
        stride,
        elements,
        buffers,
        vertices,
      })
      .collect();

    layouts.sort_by(|a, b| b.stride.cmp(&a.stride).then(b.vertices.cmp(&a.vertices)));

    layouts
  }
}
