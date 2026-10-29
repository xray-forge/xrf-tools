use serde::{Deserialize, Serialize};
use xrf_dlss::DlssRayPreset;

/// Which of DLSS ray reconstruction's models denoises and upscales: NVIDIA's pick, or one for every quality.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderRayReconstructionPreset {
  /// NVIDIA's pick, which a driver update may change; D under the physical light, whose dark foliage against a bright
  /// sky F breaks into red, green and blue blotches.
  #[default]
  Default,
  /// DLSS 4's transformer model, the one games ship.
  D,
  /// A further model of DLSS 4's, which NVIDIA's guides do not describe.
  E,
  /// DLSS 4.5's second-generation transformer model, NVIDIA's pick now.
  F,
}

impl RenderRayReconstructionPreset {
  /// The render preset hint NGX takes for it, the light physical or not (`is_light_physical`): F, NVIDIA's pick, breaks
  /// the physical light's dark canopy against a bright sky into single-colour blotches, the darker the input the more,
  /// which D and E take clean.
  pub fn to_dlss(self, is_light_physical: bool) -> DlssRayPreset {
    match self {
      Self::Default if is_light_physical => DlssRayPreset::D,
      Self::Default => DlssRayPreset::Default,
      Self::D => DlssRayPreset::D,
      Self::E => DlssRayPreset::E,
      Self::F => DlssRayPreset::F,
    }
  }
}
