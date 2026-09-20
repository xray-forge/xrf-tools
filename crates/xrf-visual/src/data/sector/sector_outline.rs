use serde::Serialize;
use xrf_level::{LevelSectorComposition, LevelVisual, LevelVisualsChunk};

use crate::data::visual::bounds::visual_bounds::VisualBounds;
use crate::pack::visual_conversion::convert_declared_bounds;

/// What one sector is and where it sits, before any of its geometry is read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorOutline {
  /// The sector, by its index in the sectors chunk.
  pub sector: u32,
  /// The visual the sector names, which is the root its drawables are reached through.
  pub root: u32,
  /// Drawables the root reaches, which is what packing the sector would pack.
  pub drawables: u32,
  /// Extent the sector declares, absent when it reaches no drawable.
  pub bounds: Option<VisualBounds>,
}

impl SectorOutline {
  /// Walks one sector and takes the extent its drawables declare.
  pub fn of(visuals: &LevelVisualsChunk, sector: u32, root: u32) -> Self {
    let composition: LevelSectorComposition = LevelSectorComposition::of(visuals, root);

    Self {
      bounds: composition
        .drawables
        .iter()
        .filter_map(|drawable| visuals.visuals.get(*drawable as usize))
        .map(Self::declared_bounds)
        .reduce(VisualBounds::merge),
      drawables: composition.drawables.len() as u32,
      root,
      sector,
    }
  }

  /// The extent every sector together covers, which is the level's own, or `None` when none declares one.
  pub fn merge_bounds(outlines: &[Self]) -> Option<VisualBounds> {
    outlines
      .iter()
      .filter_map(|outline| outline.bounds.clone())
      .reduce(VisualBounds::merge)
  }

  fn declared_bounds(visual: &LevelVisual) -> VisualBounds {
    convert_declared_bounds(&visual.header.bounding_box, &visual.header.bounding_sphere)
  }
}
