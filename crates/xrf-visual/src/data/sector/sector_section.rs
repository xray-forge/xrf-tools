use serde::Serialize;

use crate::data::sector::sector_surface::SectorSurface;
use crate::data::visual::geometry::visual_draw_range::VisualDrawRange;

/// One draw of a sector's own geometry: the indices to draw, and the surface they are drawn with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorSection {
  pub surface: SectorSurface,
  /// Drawables packed into this section, by their index in the visuals run.
  pub drawables: Vec<u32>,
  pub draw: VisualDrawRange,
}
