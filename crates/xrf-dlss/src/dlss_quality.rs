/// How much smaller than its output DLSS draws from: its quality modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DlssQuality {
  /// DLAA: antialiased at the output's size.
  Native,
  /// About two thirds of each side.
  Quality,
  /// About 58% of each side.
  Balanced,
  /// Half of each side.
  Performance,
  /// A third of each side.
  UltraPerformance,
}

#[cfg(feature = "sdk")]
impl DlssQuality {
  /// NGX's performance quality value.
  pub(crate) fn to_value(self) -> crate::ngx::bindings::NVSDK_NGX_PerfQuality_Value {
    use crate::ngx::bindings::*;

    match self {
      Self::Native => NVSDK_NGX_PerfQuality_Value_NVSDK_NGX_PerfQuality_Value_DLAA,
      Self::Quality => NVSDK_NGX_PerfQuality_Value_NVSDK_NGX_PerfQuality_Value_MaxQuality,
      Self::Balanced => NVSDK_NGX_PerfQuality_Value_NVSDK_NGX_PerfQuality_Value_Balanced,
      Self::Performance => NVSDK_NGX_PerfQuality_Value_NVSDK_NGX_PerfQuality_Value_MaxPerf,
      Self::UltraPerformance => NVSDK_NGX_PerfQuality_Value_NVSDK_NGX_PerfQuality_Value_UltraPerformance,
    }
  }
}
