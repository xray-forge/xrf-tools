use serde::{Deserialize, Serialize};

use crate::contract::render_overlay_segment::RenderOverlaySegment;

/// A helper drawn over a viewport's frame, unlit, as the raw colours it names.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RenderOverlay {
  /// Line segments in renderer space, each with a colour at either end.
  Lines {
    segments: Vec<RenderOverlaySegment>,
    /// Whether what the scene draws in front hides them.
    is_depth_tested: bool,
  },
  /// A disc in the sky where the light comes from, `size` device pixels across, following the camera and the lighting.
  Sun { color: [f32; 3], size: f32 },
  /// Points in renderer space, drawn as discs of one colour `size` device pixels across wherever they stand.
  Points {
    positions: Vec<[f32; 3]>,
    color: [f32; 3],
    size: f32,
    /// Whether what the scene draws in front hides them.
    is_depth_tested: bool,
  },
  /// Every skinned model's bones as segments, child to parent, following its pose.
  Skeleton {
    color: [f32; 3],
    /// Whether what the scene draws in front hides them.
    is_depth_tested: bool,
  },
}
