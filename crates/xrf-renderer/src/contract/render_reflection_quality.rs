use serde::{Deserialize, Serialize};

/// How hard the screen-space reflections trace: the size traced at and the steps a ray takes at most.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderReflectionQuality {
  /// Half size, 24 steps.
  Low,
  /// Half size, 32 steps.
  Medium,
  /// Half size, 48 steps.
  #[default]
  High,
  /// The frame's own size, 64 steps.
  Ultra,
}

impl RenderReflectionQuality {
  /// Every quality, cheapest first.
  pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Ultra];

  /// Steps a ray takes at most.
  pub const fn get_steps(self) -> u32 {
    match self {
      Self::Low => 24,
      Self::Medium => 32,
      Self::High => 48,
      Self::Ultra => 64,
    }
  }

  /// The frame's pixels a traced pixel stands for each way.
  pub const fn get_ratio(self) -> u32 {
    match self {
      Self::Ultra => 1,
      _ => 2,
    }
  }
}
