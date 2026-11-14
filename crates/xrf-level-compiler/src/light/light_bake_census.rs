use serde::{Deserialize, Serialize};

/// What a light bake made, for its report.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightBakeCensus {
  /// Lightmap charts.
  pub charts: usize,
  /// Lightmap pages, each written as `lmap#N_1` and `_2`.
  pub lightmaps: u32,
  /// Textures lit implicitly, such as terrain's, each written as `<texture>.dds` and `<texture>_lm.dds`.
  pub implicit_maps: usize,
}
