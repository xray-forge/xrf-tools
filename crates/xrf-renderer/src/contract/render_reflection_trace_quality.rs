use serde::{Deserialize, Serialize};

/// How often a pixel the screen leaves unanswered is traced against the level's geometry: every frame, or on its turn
/// once in a few, the last frame's answer reprojected in between; stretched where the rays asked overrun the list.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderReflectionTraceQuality {
  /// Once in eight frames.
  Low,
  /// Once in four frames.
  Medium,
  /// Once in two frames.
  #[default]
  High,
  /// Every frame.
  Ultra,
}

impl RenderReflectionTraceQuality {
  /// Every quality, cheapest first.
  pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Ultra];

  /// Frames a pixel waits between its rays.
  pub const fn get_turns(self) -> u32 {
    match self {
      Self::Low => 8,
      Self::Medium => 4,
      Self::High => 2,
      Self::Ultra => 1,
    }
  }
}
