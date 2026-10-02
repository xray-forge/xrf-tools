use serde::{Deserialize, Serialize};

/// How hard the ambient occlusion searches: XeGTAO's presets.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderAmbientOcclusionQuality {
  /// One direction, two steps each way.
  Low,
  /// Two directions, two steps.
  Medium,
  /// Three directions, three steps, `Base`'s choice.
  #[default]
  High,
  /// Six directions, three steps.
  Ultra,
}

impl RenderAmbientOcclusionQuality {
  /// Directions around the view, and steps each way along each.
  pub const fn get_search(self) -> (u32, u32) {
    match self {
      Self::Low => (1, 2),
      Self::Medium => (2, 2),
      Self::High => (3, 3),
      Self::Ultra => (6, 3),
    }
  }
}
