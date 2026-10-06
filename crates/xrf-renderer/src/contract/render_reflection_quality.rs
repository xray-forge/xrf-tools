use serde::{Deserialize, Serialize};

/// How hard the screen-space reflections trace: the size traced at and how many cells of the depth pyramid a ray may
/// cross before it gives up.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderReflectionQuality {
  /// Half size, 24 crossings.
  Low,
  /// Half size, 48 crossings.
  Medium,
  /// Half size, 96 crossings.
  #[default]
  High,
  /// The frame's own size, 128 crossings.
  Ultra,
}

impl RenderReflectionQuality {
  /// Every quality, cheapest first.
  pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Ultra];

  /// Cells of the depth pyramid a ray crosses at most before it is given up as having met nothing.
  pub const fn get_crossings(self) -> u32 {
    match self {
      Self::Low => 24,
      Self::Medium => 48,
      Self::High => 96,
      Self::Ultra => 128,
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
