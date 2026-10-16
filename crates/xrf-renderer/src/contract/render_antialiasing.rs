use serde::{Deserialize, Serialize};

/// How a viewport's finished frame has its edges smoothed.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderAntialiasing {
  /// Every edge as drawn.
  #[default]
  None,
  /// One pass over the drawn frame's edges, softest and cheapest.
  Fxaa,
  /// Three passes over the drawn frame's edges, crisp and stable.
  Smaa,
  /// Temporal: every frame's samples jittered within the pixel and resolved with the frames before.
  Taa,
  /// FSR 3.1: AMD's temporal upscaler, temporal as TAA with each surface's motion, disocclusion, shading change,
  /// reactivity and thin-feature locks.
  Fsr,
  /// DLSS: NVIDIA's learned temporal upscaler, on an RTX GPU; FSR where the GPU has none.
  Dlss,
}

impl RenderAntialiasing {
  /// Whether it resolves jittered frames with their history, which jitters every scene pass.
  pub fn is_temporal(self) -> bool {
    matches!(self, Self::Taa | Self::Fsr | Self::Dlss)
  }

  /// What it smooths with on a GPU that has DLSS where `has_dlss`: DLSS falls back to FSR where it has none.
  pub fn get_available(self, has_dlss: bool) -> Self {
    match self {
      Self::Dlss if !has_dlss => Self::Fsr,
      other => other,
    }
  }
}
