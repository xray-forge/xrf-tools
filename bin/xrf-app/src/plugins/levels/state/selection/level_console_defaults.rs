use serde::Serialize;

/// What a game's shipped console defaults (Anomaly's `default_controls.ltx`) set of how its levels are lit, exposed and
/// corrected; each `None` it leaves to the engine.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelConsoleDefaults {
  /// Whether the game ships console defaults at all.
  pub is_shipped: bool,
  /// `r2_tonemap`.
  pub is_tonemapped: Option<bool>,
  /// `r2_tonemap_amount`, `r2_tonemap_middlegray`, `r2_tonemap_lowlum` and `r2_tonemap_adaptation`.
  pub tonemap_amount: Option<f32>,
  pub tonemap_middle_gray: Option<f32>,
  pub tonemap_low_luminance: Option<f32>,
  pub tonemap_adaptation: Option<f32>,
  /// `r2_sun_lumscale`, `r2_sun_lumscale_hemi` and `r2_sun_lumscale_amb`.
  pub sun_scale: Option<f32>,
  pub hemi_scale: Option<f32>,
  pub ambient_scale: Option<f32>,
  /// `r__exposure`, `r__gamma`, `r__saturation` and `r__color_grading`.
  pub image_exposure: Option<f32>,
  pub image_gamma: Option<f32>,
  pub image_saturation: Option<f32>,
  pub color_grading: Option<[f32; 3]>,
}
