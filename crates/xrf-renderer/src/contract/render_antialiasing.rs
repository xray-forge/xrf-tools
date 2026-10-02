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
  /// FSR 2: temporal as TAA, with each surface's motion, disocclusion, reactivity and thin-feature locks.
  Fsr2,
}

impl RenderAntialiasing {
  /// Whether it resolves jittered frames with their history, which jitters every scene pass.
  pub fn is_temporal(self) -> bool {
    matches!(self, Self::Taa | Self::Fsr2)
  }
}
