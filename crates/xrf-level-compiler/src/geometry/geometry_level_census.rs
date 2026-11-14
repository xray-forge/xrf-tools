use serde::{Deserialize, Serialize};

/// What a geometry compile made, for its report.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryLevelCensus {
  /// Static visuals.
  pub visuals: usize,
  /// Their triangles at full detail.
  pub triangles: usize,
  /// Static and multi-use visuals made progressive.
  pub progressive: usize,
  /// Multi-use placements.
  pub trees: usize,
  pub sectors: usize,
  /// Lightmap charts.
  pub charts: usize,
  /// Lightmap pages, each written as `lmap#N_1` and `_2`.
  pub lightmaps: u32,
}
