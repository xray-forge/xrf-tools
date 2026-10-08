use serde::{Deserialize, Serialize};

/// How hard the ambient occlusion searches: XeGTAO's presets, and VBAO's own.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderAmbientOcclusionQuality {
  /// GTAO one direction, two steps each way; VBAO three steps.
  Low,
  /// GTAO two directions, two steps; VBAO five steps.
  Medium,
  /// GTAO three directions, three steps; VBAO eight steps. `Base`'s choice.
  #[default]
  High,
  /// GTAO six directions, three steps; VBAO two directions, eight steps.
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

  /// VBAO's directions and steps each way: one direction, turned every frame its accumulation gathers, and more
  /// steps, since a thin occluder covers only the angles its thickness reaches.
  pub const fn get_vbao_search(self) -> (u32, u32) {
    match self {
      Self::Low => (1, 3),
      Self::Medium => (1, 5),
      Self::High => (1, 8),
      Self::Ultra => (2, 8),
    }
  }
}
