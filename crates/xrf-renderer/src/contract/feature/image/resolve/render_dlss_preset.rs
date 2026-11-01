use serde::{Deserialize, Serialize};
use xrf_dlss::DlssPreset;

/// Which of DLSS's models upscales: NVIDIA's pick for each quality, or one for every quality.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDlssPreset {
  /// NVIDIA's pick, which a driver update may change: K for DLAA, Quality and Balanced, M for Performance, L for Ultra
  /// Performance.
  #[default]
  Default,
  /// Transformer, like K, with slightly less ghosting and more flicker.
  J,
  /// Transformer: the best image, at the highest cost.
  K,
  /// NVIDIA's model for Ultra Performance.
  L,
  /// NVIDIA's model for Performance.
  M,
}

impl RenderDlssPreset {
  /// The render preset hint NGX takes for it.
  pub fn to_dlss(self) -> DlssPreset {
    match self {
      Self::Default => DlssPreset::Default,
      Self::J => DlssPreset::J,
      Self::K => DlssPreset::K,
      Self::L => DlssPreset::L,
      Self::M => DlssPreset::M,
    }
  }
}
