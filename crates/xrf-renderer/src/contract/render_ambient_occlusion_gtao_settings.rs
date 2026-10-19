use serde::{Deserialize, Serialize};

/// GTAO's search: the slices it turns about a pixel and the steps it takes each way along each.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAmbientOcclusionGtaoSettings {
  pub slices: u32,
  pub steps: u32,
}

impl Default for RenderAmbientOcclusionGtaoSettings {
  /// XeGTAO's High: three slices of three steps.
  fn default() -> Self {
    Self { slices: 3, steps: 3 }
  }
}

impl RenderAmbientOcclusionGtaoSettings {
  /// The least and most slices, and steps, it searches.
  pub const SLICES_RANGE: (u32, u32) = (1, 9);
  pub const STEPS_RANGE: (u32, u32) = (1, 6);

  pub fn get_slices(&self) -> u32 {
    self.slices.clamp(Self::SLICES_RANGE.0, Self::SLICES_RANGE.1)
  }

  pub fn get_steps(&self) -> u32 {
    self.steps.clamp(Self::STEPS_RANGE.0, Self::STEPS_RANGE.1)
  }
}
