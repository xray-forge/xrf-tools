use serde::{Deserialize, Serialize};
use xrf_dlss::DlssRayPreset;

/// Which of DLSS ray reconstruction's models denoises and upscales: NVIDIA's pick, or one for every quality.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderRayReconstructionPreset {
  /// NVIDIA's pick, which a driver update may change.
  #[default]
  Default,
  /// The transformer model.
  D,
  /// The latest transformer model.
  E,
  /// NVIDIA's default model, RR2.
  F,
}

impl RenderRayReconstructionPreset {
  /// The render preset hint NGX takes for it.
  pub fn to_dlss(self) -> DlssRayPreset {
    match self {
      Self::Default => DlssRayPreset::Default,
      Self::D => DlssRayPreset::D,
      Self::E => DlssRayPreset::E,
      Self::F => DlssRayPreset::F,
    }
  }
}
