use serde::Serialize;

/// What a game's console sets of how its levels are lit, exposed and corrected: its installation's `user.ltx` over its
/// shipped defaults (Anomaly's `default_controls.ltx`); each `None` it leaves to the engine.
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
  /// `r2_ls_bloom_threshold`, `r2_ls_bloom_kernel_g` and `r2_ls_bloom_kernel_scale`.
  pub bloom_threshold: Option<f32>,
  pub bloom_radius: Option<f32>,
  pub bloom_strength: Option<f32>,
}

impl LevelConsoleDefaults {
  /// These commands over `base`'s: each one these leave unset taken from it.
  pub fn over(self, base: Self) -> Self {
    Self {
      is_shipped: self.is_shipped || base.is_shipped,
      is_tonemapped: self.is_tonemapped.or(base.is_tonemapped),
      tonemap_amount: self.tonemap_amount.or(base.tonemap_amount),
      tonemap_middle_gray: self.tonemap_middle_gray.or(base.tonemap_middle_gray),
      tonemap_low_luminance: self.tonemap_low_luminance.or(base.tonemap_low_luminance),
      tonemap_adaptation: self.tonemap_adaptation.or(base.tonemap_adaptation),
      sun_scale: self.sun_scale.or(base.sun_scale),
      hemi_scale: self.hemi_scale.or(base.hemi_scale),
      ambient_scale: self.ambient_scale.or(base.ambient_scale),
      image_exposure: self.image_exposure.or(base.image_exposure),
      image_gamma: self.image_gamma.or(base.image_gamma),
      image_saturation: self.image_saturation.or(base.image_saturation),
      color_grading: self.color_grading.or(base.color_grading),
      bloom_threshold: self.bloom_threshold.or(base.bloom_threshold),
      bloom_radius: self.bloom_radius.or(base.bloom_radius),
      bloom_strength: self.bloom_strength.or(base.bloom_strength),
    }
  }
}
