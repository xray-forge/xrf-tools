use crate::chart::lightmap_layer::LightmapLayer;
use crate::chart::lit_chart::LitChart;

/// Where a chart stands once first lit and filled ([`crate::LightChartLighter::settle`]): lit already, one colour; to
/// be lit again at a smaller size; or to take its border as it is.
pub(crate) enum LitChartNext {
  Lit(LitChart),
  Relit(u32, u32),
  Bordered(LightmapLayer),
}
