use crate::light::lightmap::lit::lightmap_layer::LightmapLayer;

/// A chart lit and bordered ([`crate::light::lightmap::lit::light_chart_lighter::LightChartLighter`]): its texels a
/// side as lit, 0 for a side compressed to one colour or one texel across, and its layer with the border, a texel wider
/// each side, or two texels across for such a side.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LitChart {
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) layer: LightmapLayer,
}
