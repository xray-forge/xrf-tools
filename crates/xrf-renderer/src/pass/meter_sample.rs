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
}

impl MeterSample {
  /// What a reading says of it: the surfaces' values only where the spot covers any.
  pub fn to_reading(self) -> RenderMeterReading {
    let surface = |value: f32| (self.drawn > 0.0).then_some(value);

    RenderMeterReading {
      luminance: self.luminance,
      stops: self.stops,
      exposure: self.exposure,
      albedo: surface(self.albedo),
      gloss: surface(self.gloss),
      diffuse: surface(self.diffuse),
      specular: surface(self.specular),
      distance: surface(self.distance),
    }
  }
}
