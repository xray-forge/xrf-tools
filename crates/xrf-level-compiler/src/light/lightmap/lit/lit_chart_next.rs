use crate::light::lightmap::lit::lightmap_layer::LightmapLayer;
use crate::light::lightmap::lit::lit_chart::LitChart;

/// Where a chart stands once first lit and filled
/// ([`crate::light::lightmap::lit::light_chart_lighter::LightChartLighter::settle`]): lit already, one colour; to be
/// lit again at a smaller size, its texels a side; or to take its border as it is.
pub(crate) enum LitChartNext {
  Lit(LitChart),
  Relit(u32, u32),
  Bordered(LightmapLayer),
}
