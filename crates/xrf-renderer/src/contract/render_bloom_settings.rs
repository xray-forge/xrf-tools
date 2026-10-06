use serde::{Deserialize, Serialize};

/// The engine's bloom (`phase_bloom`): the bright part of the frame blurred over it, as the console sets it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderBloomSettings {
  pub is_enabled: bool,
  /// `r2_ls_bloom_threshold`: what the summed brightness loses before it scales the blur.
  pub threshold: f32,
  /// `r2_ls_bloom_kernel_g`: the blur's radius, in texels of its 256-square target.
  pub radius: f32,
  /// `r2_ls_bloom_kernel_scale`: how strong the blur is.
  pub strength: f32,
}

impl RenderBloomSettings {
  /// OpenXRay's console defaults, which bloom the whole frame.
  pub const OPENXRAY: Self = Self {
    is_enabled: true,
    threshold: 0.00001,
    radius: 3.0,
    strength: 0.7,
  };
}

impl Default for RenderBloomSettings {
  /// None: a scene the level's look does not bloom.
  fn default() -> Self {
    Self {
      is_enabled: false,
      ..Self::OPENXRAY
    }
  }
}
