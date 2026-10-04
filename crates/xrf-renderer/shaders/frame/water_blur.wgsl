#import "common/fullscreen"

// Screen Space Shaders' `ssfx_water_blur.ps`: one way of the enhanced water's reflection blurred into a target half the
// reflection's size, thirteen taps spaced by half-size texels, mixed with the reflection as it was by the blur's share.

// The part of the water's uniform the blur reads, laid as `static/water.wgsl` lays it.
struct Water {
  time: f32,
  wave_height: f32,
  wave_speed: f32,
  ripple: f32,
  reflection: f32,
  intensity: f32,
  soft: f32,
  distorted: f32,
  refraction: f32,
  turbidity: f32,
  soft_border: f32,
  reflectivity: f32,
  reflection_blur: f32,
  blur_noise: f32,
  reflected: f32,
  history: f32,
  wind_direction: f32,
  wind_velocity: f32,
  specular: f32,
  caustics: f32,
};

// xy: the way it blurs; z: how many times the target's size the source's is.
struct WaterBlur {
  direction: vec4<f32>,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> water: Water;
@group(0) @binding(3) var<uniform> blur: WaterBlur;

// Taps either side of the centre.
const TAPS: i32 = 6;

@fragment
fn fs_water_blur(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(source));
  let scale: f32 = blur.direction.z;
  let uv: vec2<f32> = in.clip.xy * scale / size;
  let texel: vec2<f32> = scale / size;
  let offset: vec2<f32> = blur.direction.xy * texel * clamp(water.reflection_blur * 2.0, 1.0, 2.0);
  let base: vec3<f32> = textureSampleLevel(source, source_sampler, uv, 0.0).rgb;
  var sum: vec3<f32> = vec3<f32>(0.0);

  for (var tap: i32 = -TAPS; tap <= TAPS; tap++) {
    sum += textureSampleLevel(source, source_sampler, clamp(uv + f32(tap) * offset, vec2<f32>(0.0), vec2<f32>(1.0)), 0.0)
      .rgb;
  }

  // The module divides its thirteen taps by twelve, which brightens the blur a little; kept as it is.
  return vec4<f32>(mix(base, sum / 12.0, water.reflection_blur), 1.0);
}
