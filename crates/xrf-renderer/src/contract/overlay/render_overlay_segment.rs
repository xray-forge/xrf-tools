use serde::{Deserialize, Serialize};

/// One line segment of an overlay: its two ends in renderer space, and the raw colour at each.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderOverlaySegment {
  pub ends: [[f32; 3]; 2],
  pub colors: [[f32; 3]; 2],
}
