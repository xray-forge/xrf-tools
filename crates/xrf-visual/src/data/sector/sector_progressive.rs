use serde::Serialize;

use crate::data::visual::geometry::visual_draw_range::VisualDrawRange;

/// The bands a progressive mesh is drawn in: a few of the engine's slide windows, one of which each place draws.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorProgressive {
  /// Windows the engine's table has, which a place's detail picks among (`FTreeVisual_PM::Render`).
  pub windows: u32,
  /// A range of the mesh's indices per band, the whole detail first. Band `b` is window `floor(b * windows / bands)`,
  /// so a place drawing the band its window falls in is never coarser than the engine would draw it.
  pub bands: Vec<VisualDrawRange>,
}

impl SectorProgressive {
  /// Bands a mesh is split into at most: each is a draw of its own, and windows further apart than this differ by
  /// a few triangles.
  pub const MAX_BANDS: u32 = 4;

  /// The window band `band` of `bands` draws, out of `windows`.
  pub const fn get_band_window(band: u32, bands: u32, windows: u32) -> u32 {
    band * windows / bands
  }
}
