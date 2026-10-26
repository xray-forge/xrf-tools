use serde::{Deserialize, Serialize};

/// One parameter of a post-process effect, `pp_params`, in the order the file stores them and the engine numbers them.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PpeParameter {
  /// `pp_base_color`.
  BaseColor,
  /// `pp_add_color`.
  AddColor,
  /// `pp_gray_color`.
  GrayColor,
  /// `pp_gray_value`.
  GrayValue,
  /// `pp_blur`.
  Blur,
  /// `pp_dual_h`.
  DualityHorizontal,
  /// `pp_dual_v`.
  DualityVertical,
  /// `pp_noise_i`.
  NoiseIntensity,
  /// `pp_noise_g`.
  NoiseGranularity,
  /// `pp_noise_f`.
  NoiseFps,
  /// `pp_cm_influence`, which only a version 2 effect stores.
  ColorMapInfluence,
}

impl PpeParameter {
  /// Every parameter, in the order the file stores them.
  pub const ALL: [Self; 11] = [
    Self::BaseColor,
    Self::AddColor,
    Self::GrayColor,
    Self::GrayValue,
    Self::Blur,
    Self::DualityHorizontal,
    Self::DualityVertical,
    Self::NoiseIntensity,
    Self::NoiseGranularity,
    Self::NoiseFps,
    Self::ColorMapInfluence,
  ];

  /// The name an author knows the parameter by.
  pub const fn get_name(self) -> &'static str {
    match self {
      Self::BaseColor => "base color",
      Self::AddColor => "add color",
      Self::GrayColor => "gray color",
      Self::GrayValue => "gray value",
      Self::Blur => "blur",
      Self::DualityHorizontal => "duality horizontal",
      Self::DualityVertical => "duality vertical",
      Self::NoiseIntensity => "noise intensity",
      Self::NoiseGranularity => "noise granularity",
      Self::NoiseFps => "noise fps",
      Self::ColorMapInfluence => "colour map influence",
    }
  }

  /// Whether the parameter is a colour of three channels (`CPostProcessColor`) rather than one value.
  pub const fn is_color(self) -> bool {
    matches!(self, Self::BaseColor | Self::AddColor | Self::GrayColor)
  }
}
