// The frame's lighting, as `pass/lighting_uniform.rs` writes it, and the engine's own lighting functions.

struct Lighting {
  // xyz: towards the sun, in view space.
  to_sun: vec4<f32>,
  // rgb: the sun's colour; w: its specular weight.
  sun: vec4<f32>,
  ambient: vec4<f32>,
  environment: vec4<f32>,
  sky_irradiance: vec4<f32>,
  sky_zenith: vec4<f32>,
  sky_horizon: vec4<f32>,
  // x: the settings' tonemap scale; y: one where the scene is lit; z: one where the exposure adapts; w: one where the
  // ambient occlusion darkens the hemisphere.
  params: vec4<f32>,
};

// `fWhiteIntensity` of `tonemap`, squared.
const WHITE_INTENSITY_SQUARED: f32 = 1.7 * 1.7;

// `tonemap` of `common_functions.h`: the engine's curve, `x (1 + x / W²) / (1 + x)` over the scaled colour.
fn tonemap(color: vec3<f32>, scale: f32) -> vec3<f32> {
  let x: vec3<f32> = color * scale;

  return x * (1.0 + x / WHITE_INTENSITY_SQUARED) / (1.0 + x);
}

// The scaled colour `tonemap` took to a tonemapped one: the curve solved for `x`.
fn untonemap(color: vec3<f32>) -> vec3<f32> {
  let rest: vec3<f32> = 1.0 - color;

  return (sqrt(rest * rest + color * (4.0 / WHITE_INTENSITY_SQUARED)) - rest) * (WHITE_INTENSITY_SQUARED / 2.0);
}
