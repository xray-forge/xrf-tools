use xrf_renderer_core::ShaderStruct;

use crate::contract::render_exposure_settings::RenderExposureSettings;

/// How the physical light's exposure meters and adapts, as `shaders/frame/exposure_histogram.wgsl` reads it: the log2
/// luminance the histogram's bins span, the luminance the frame is brought to, how much of a scene's distance from a
/// daylit exterior it keeps, how far it may move this frame each way, the share of the darkest and the brightest light
/// the average leaves out, and the scale's bounds.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "ExposureHistogram")]
pub struct ExposureHistogramUniform {
  pub least_log: f32,
  pub span_log: f32,
  pub aim: f32,
  pub reference_log: f32,
  pub kept_share: f32,
  pub delta: f32,
  pub brightening: f32,
  pub darkening: f32,
  pub low_share: f32,
  pub high_share: f32,
  pub least_scale_log: f32,
  pub most_scale_log: f32,
  /// One where the scale goes straight to its target, as on the first frame metered.
  pub snap: f32,
}

impl ExposureHistogramUniform {
  /// Bins the frame's log2 luminance is counted in.
  pub const BINS: u32 = 128;
  /// The luminance the bins span, log2: about 1/4096 to 64 in the scene's units.
  pub const LEAST_LOG: f32 = -12.0;
  pub const MOST_LOG: f32 = 6.0;
  /// The luminance the average is brought to: middle grey.
  pub const MIDDLE_GREY: f32 = 0.18;
  /// The log2 luminance of a daylit exterior, which is shown at the aim; a darker or brighter scene keeps this share of
  /// its distance from it, so a cellar stays darker than noon and snow brighter, as the eye adapts only part of the way.
  pub const REFERENCE_LOG: f32 = -3.0;
  pub const KEPT_SHARE: f32 = 0.5;
  /// Stops a second the scale moves: faster darkening a brighter frame than brightening a darker one, as the eye does.
  pub const DARKENING: f32 = 3.0;
  pub const BRIGHTENING: f32 = 1.0;
  /// The darkest and the brightest shares of the frame the average leaves out.
  pub const LOW_SHARE: f32 = 0.1;
  pub const HIGH_SHARE: f32 = 0.9;
  /// The scale's bounds, log2.
  pub const LEAST_SCALE_LOG: f32 = -10.0;
  pub const MOST_SCALE_LOG: f32 = 10.0;

  /// The metering for a frame `delta` seconds after the last, straight to its target where `is_snapped`.
  pub fn new(settings: &RenderExposureSettings, delta: f32, is_snapped: bool) -> Self {
    Self {
      least_log: Self::LEAST_LOG,
      span_log: Self::MOST_LOG - Self::LEAST_LOG,
      aim: Self::MIDDLE_GREY * settings.compensation.exp2(),
      reference_log: Self::REFERENCE_LOG,
      kept_share: Self::KEPT_SHARE,
      delta,
      brightening: Self::BRIGHTENING,
      darkening: Self::DARKENING,
      low_share: Self::LOW_SHARE,
      high_share: Self::HIGH_SHARE,
      least_scale_log: Self::LEAST_SCALE_LOG,
      most_scale_log: Self::MOST_SCALE_LOG,
      snap: f32::from(u8::from(is_snapped)),
    }
  }
}
