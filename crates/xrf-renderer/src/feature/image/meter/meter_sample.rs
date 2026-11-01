use xrf_renderer_core::ShaderStruct;

use crate::contract::render_meter_reading::RenderMeterReading;

/// What the spot meter writes on the GPU, as `shaders/frame/meter.wgsl` reads it: the light at the spot and what the
/// G-buffer holds there, `drawn` the share of its texels any surface covers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "MeterSample")]
pub struct MeterSample {
  pub luminance: f32,
  pub stops: f32,
  pub exposure: f32,
  pub drawn: f32,
  pub albedo: f32,
  pub gloss: f32,
  pub diffuse: f32,
  pub specular: f32,
  pub distance: f32,
  /// One where the light is physical, its luminance in cd/m².
  pub photometric: f32,
}

/// The reflected-light meter's calibration constant, `K` of ISO 2720, as camera makers set it.
const REFLECTED_CALIBRATION: f32 = 12.5;

impl MeterSample {
  /// What a reading says of it: the surfaces' values only where the spot covers any.
  pub fn to_reading(self) -> RenderMeterReading {
    let surface = |value: f32| (self.drawn > 0.0).then_some(value);

    RenderMeterReading {
      luminance: self.luminance,
      stops: self.stops,
      exposure: self.exposure,
      ev100: (self.photometric > 0.5).then(|| (self.luminance.max(1e-6) * 100.0 / REFLECTED_CALIBRATION).log2()),
      albedo: surface(self.albedo),
      gloss: surface(self.gloss),
      diffuse: surface(self.diffuse),
      specular: surface(self.specular),
      distance: surface(self.distance),
    }
  }
}
