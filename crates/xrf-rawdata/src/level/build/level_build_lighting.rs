use xrf_editable_level::{EditableBuildParams, EditableLevelOptions};

/// The lighting a build hands xrLC in place of the scene's own: the lightmaps' pixels per metre and jitter samples
/// (`build_params`), and the sky's and the sun's quality (`level_options`), which together set how long xrLC lights.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelBuildLighting {
  pub pixels_per_meter: f32,
  pub jitter_samples: u32,
  pub hemi_quality: u8,
  pub sun_quality: u8,
}

impl LevelBuildLighting {
  /// A proof build's, to check a level rather than ship it: 3 pixels a metre, one sample, the sky and the sun at quality
  /// 1. Vanilla `jupiter` took xrLC 40 minutes so, against hours at the decompile's 10 pixels and 9 samples.
  pub const PROOF: Self = Self {
    pixels_per_meter: 3.0,
    jitter_samples: 1,
    hemi_quality: 1,
    sun_quality: 1,
  };

  /// Sets the scene's `params` and `options` to this lighting, leaving every other setting as the scene has it.
  pub fn apply(&self, params: &mut EditableBuildParams, options: &mut EditableLevelOptions) {
    params.light_pixel_per_meter = self.pixels_per_meter;
    params.light_jitter_samples = self.jitter_samples;
    options.light_hemi_quality = self.hemi_quality;
    options.light_sun_quality = self.sun_quality;
  }
}
