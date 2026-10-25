/// Which of NGX's ray reconstruction models runs: NVIDIA's pick, or one asked for every quality.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DlssRayPreset {
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

#[cfg(feature = "sdk")]
impl DlssRayPreset {
  /// The value of NGX's ray reconstruction render preset hint.
  pub(crate) fn to_hint(self) -> crate::ngx::bindings::NVSDK_NGX_RayReconstruction_Hint_Render_Preset {
    use crate::ngx::bindings::*;

    match self {
      Self::Default => {
        NVSDK_NGX_RayReconstruction_Hint_Render_Preset_NVSDK_NGX_RayReconstruction_Hint_Render_Preset_Default
      }
      Self::D => NVSDK_NGX_RayReconstruction_Hint_Render_Preset_NVSDK_NGX_RayReconstruction_Hint_Render_Preset_D,
      Self::E => NVSDK_NGX_RayReconstruction_Hint_Render_Preset_NVSDK_NGX_RayReconstruction_Hint_Render_Preset_E,
      Self::F => NVSDK_NGX_RayReconstruction_Hint_Render_Preset_NVSDK_NGX_RayReconstruction_Hint_Render_Preset_F,
    }
  }
}
