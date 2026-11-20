#import "common/fullscreen"
#import "common/water_reflection_read"
#import "generated/frame/water_blur"

// One way of the enhanced water's reflection blurred into a target half the reflection's size, thirteen taps spaced by
// half-size texels, mixed with the reflection as it was by the blur's share: of the water's texels alone, so nothing
// standing in the water darkens the reflection about it.

// Taps either side of the centre, which `WaterBlurUniform::REACH` counts to keep each way's rectangles.
const TAPS: i32 = 6;

@fragment
fn fs_water_blur(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(source));
  let scale: f32 = blur.direction.z;
  let uv: vec2<f32> = in.clip.xy * scale / size;
  let texel: vec2<f32> = scale / size;
  let offset: vec2<f32> = blur.direction.xy * texel * clamp(enhanced.reflection_blur * 2.0, 1.0, 2.0);
  let base: vec4<f32> = read_water_reflection(source, uv);
  var sum: vec3<f32> = vec3<f32>(0.0);
  var weight: f32 = 0.0;

  for (var tap: i32 = -TAPS; tap <= TAPS; tap++) {
    let read: vec4<f32> = read_water_reflection(source, clamp(uv + f32(tap) * offset, vec2<f32>(0.0), vec2<f32>(1.0)));

    sum += read.rgb * read.a;
    weight += read.a;
  }

  // Nothing where no water stands about it, which the next way and the surface take for no water.
  if (weight <= 0.0) {
    return vec4<f32>(0.0);
  }

  // The water's taps averaged, then by thirteen twelfths, as thirteen taps divided by twelve brightened it a little.
  let blurred: vec3<f32> = sum / weight * (13.0 / 12.0);

  return vec4<f32>(mix(select(blurred, base.rgb, base.a > 0.0), blurred, enhanced.reflection_blur), 1.0);
}
