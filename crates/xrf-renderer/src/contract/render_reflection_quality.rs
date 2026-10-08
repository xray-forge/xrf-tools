use serde::{Deserialize, Serialize};

/// How hard the screen-space reflections trace: the size traced at, the steps a ray takes at most, how far behind a
/// surface a step may land and still have met it, and whether a step behind one is halved back to where it crossed.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderReflectionQuality {
  /// Half size, 16 steps.
  Low,
  /// Half size, 24 steps.
  Medium,
  /// Half size, 32 steps, refined.
  #[default]
  High,
  /// The frame's own size, 64 steps, refined.
  Ultra,
}

impl RenderReflectionQuality {
  /// Every quality, cheapest first.
  pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Ultra];

  /// Steps a ray takes at most.
  pub const fn get_steps(self) -> u32 {
    match self {
      Self::Low => 16,
      Self::Medium => 24,
      Self::High => 32,
      Self::Ultra => 64,
    }
  }

  /// Metres behind a surface an unrefined step may land and still have met it: the longer the steps, the further.
  pub const fn get_limit(self) -> f32 {
    match self {
      Self::Low => 36.0,
      Self::Medium => 18.0,
      Self::High => 5.0,
      Self::Ultra => 1.0,
    }
  }

  /// Whether a step behind a surface is halved back to where it crossed, which then holds it to a thin surface.
  pub const fn is_refined(self) -> bool {
    matches!(self, Self::High | Self::Ultra)
  }

  /// The frame's pixels a traced pixel stands for each way.
  pub const fn get_ratio(self) -> u32 {
    match self {
      Self::Ultra => 1,
      _ => 2,
    }
  }
}
