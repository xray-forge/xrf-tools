use crate::contract::render_exposure_settings::RenderExposureSettings;

/// `MiddleGray`, as `shaders/frame/exposure.wgsl` declares it: the scale is `target / (luminance * weight + floor)`,
/// moved towards by `blend` of the way.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ExposureUniform {
  pub target: f32,
  pub weight: f32,
  pub floor: f32,
  pub blend: f32,
}

impl ExposureUniform {
  /// `(1, 0, 1)` lerped towards `(middle gray, 1, low luminance)` by the amount, as `phase_luminance` sets it.
  pub fn new(settings: &RenderExposureSettings, blend: f32) -> Self {
    let amount: f32 = settings.amount;

    Self {
      target: 1.0 + (settings.middle_gray - 1.0) * amount,
      weight: amount,
      floor: 1.0 + (settings.low_luminance - 1.0) * amount,
      blend,
    }
  }
}
