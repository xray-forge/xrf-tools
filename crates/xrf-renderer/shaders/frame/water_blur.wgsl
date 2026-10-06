#import "common/fullscreen"
#import "generated/frame/water_blur"

// Screen Space Shaders' `ssfx_water_blur.ps`: one way of the enhanced water's reflection blurred into a target half the
// reflection's size, thirteen taps spaced by half-size texels, mixed with the reflection as it was by the blur's share.

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
