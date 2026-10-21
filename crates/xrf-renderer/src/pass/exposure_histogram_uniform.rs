use xrf_renderer_core::ShaderStruct;

use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::contract::render_light_level::RenderLightLevel;

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
  /// The log2 luminance the frame's average is never shown under: the dark's limit.
  pub darkest_log: f32,
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
  /// The luminance the bins span, log2: about 1/16 000 cd/m², under starlight, to 65 000, sunlit snow and past it.
  pub const LEAST_LOG: f32 = -14.0;
  pub const MOST_LOG: f32 = 16.0;
  /// The luminance the average is brought to: middle grey.
  pub const MIDDLE_GREY: f32 = 0.18;
  /// The log2 luminance of a daylit exterior's average, about 1 260 cd/m², which is shown at the aim; a darker or
  /// brighter scene keeps this share of its distance from it, so a cellar stays darker than noon and snow brighter, as
  /// the eye adapts only part of the way: two thirds, which shows a shaded dawn as darkly as the engine's meter does.
  pub const REFERENCE_LOG: f32 = 10.3;
  pub const KEPT_SHARE: f32 = 0.67;
  /// Stops under the aim the physical light levels show the darkest scene at most, so a moonlit night shows dark but
  /// readable; the authored levels' dark never reaches it.
  pub const PHYSICAL_DARKEST_STOPS: f32 = 4.0;
  /// Stops a second the scale moves: faster darkening a brighter frame than brightening a darker one, as the eye does,
  /// and quick enough that a night under the physical levels, eleven stops under noon, settles in about three seconds.
  pub const DARKENING: f32 = 12.0;
  pub const BRIGHTENING: f32 = 4.0;
  /// The darkest and the brightest shares of the frame the average leaves out.
  pub const LOW_SHARE: f32 = 0.1;
  pub const HIGH_SHARE: f32 = 0.9;
  /// The scale's bounds, log2: middle grey shown at about 47 000 cd/m², or at about 1/1 400.
  pub const LEAST_SCALE_LOG: f32 = -18.0;
  pub const MOST_SCALE_LOG: f32 = 8.0;

  /// The scale a daylit exterior is shown at, its average at the aim, raised by `compensation` stops: the physical
  /// light's first and the one it holds while it does not adapt.
  pub fn get_daylight_scale(compensation: f32) -> f32 {
    (Self::MIDDLE_GREY.log2() + compensation - Self::REFERENCE_LOG).exp2()
  }

  /// The metering for a frame `delta` seconds after the last at the light's `level`, straight to its target where
  /// `is_snapped`.
  pub fn new(settings: &RenderExposureSettings, level: RenderLightLevel, delta: f32, is_snapped: bool) -> Self {
    let aim: f32 = Self::MIDDLE_GREY * settings.compensation.exp2();

    Self {
      least_log: Self::LEAST_LOG,
      span_log: Self::MOST_LOG - Self::LEAST_LOG,
      aim,
      reference_log: Self::REFERENCE_LOG,
      kept_share: Self::KEPT_SHARE,
      darkest_log: match level {
        RenderLightLevel::Authored => Self::LEAST_LOG,
        RenderLightLevel::Physical => aim.log2() - Self::PHYSICAL_DARKEST_STOPS,
      },
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
