use serde::{Deserialize, Serialize};

use crate::geom::level_geom_slide_window::LevelGeomSlideWindow;

/// One progressive mesh's chain of detail levels, `FSlideWindowItem` (`xrCore/FMesh.hpp`).
///
/// Usually a tree: `CRender::LoadSWIs` reads these so a visual can drop detail with distance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGeomSlideWindowItem {
  /// Four words the loader reads into `FSlideWindowItem::reserved` and never looks at again.
  pub reserved: [u32; 4],
  /// The detail levels, finest first, as the renderer indexes them.
  pub windows: Vec<LevelGeomSlideWindow>,
}

impl LevelGeomSlideWindowItem {
  /// Bytes the fixed head of a record occupies, before its own detail levels.
  pub const HEADER_SIZE: u64 = 4 * 4 + 4;
}
