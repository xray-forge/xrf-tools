/// Which of NGX's super resolution models runs: NVIDIA's pick for each quality, or one asked for every quality.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DlssPreset {
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

#[cfg(feature = "sdk")]
impl DlssPreset {
  /// The value of NGX's render preset hint.
  pub(crate) fn to_hint(self) -> crate::ngx::bindings::NVSDK_NGX_DLSS_Hint_Render_Preset {
    use crate::ngx::bindings::*;

    match self {
      Self::Default => NVSDK_NGX_DLSS_Hint_Render_Preset_NVSDK_NGX_DLSS_Hint_Render_Preset_Default,
      Self::J => NVSDK_NGX_DLSS_Hint_Render_Preset_NVSDK_NGX_DLSS_Hint_Render_Preset_J,
      Self::K => NVSDK_NGX_DLSS_Hint_Render_Preset_NVSDK_NGX_DLSS_Hint_Render_Preset_K,
      Self::L => NVSDK_NGX_DLSS_Hint_Render_Preset_NVSDK_NGX_DLSS_Hint_Render_Preset_L,
      Self::M => NVSDK_NGX_DLSS_Hint_Render_Preset_NVSDK_NGX_DLSS_Hint_Render_Preset_M,
    }
  }
}
