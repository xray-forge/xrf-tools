use crate::chart::lightmap_layer::LightmapLayer;

/// A chart lit and bordered ([`crate::LightChartLighter`]): its texels a side as lit, 0 for a side compressed to one
/// colour or one texel across, and its layer with the border, a texel wider each side, or two texels across for such a
/// side.
#[derive(Clone, Debug, PartialEq)]
pub struct LitChart {
  pub width: u32,
  pub height: u32,
  pub layer: LightmapLayer,
}
