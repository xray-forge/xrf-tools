#import "common/fullscreen"
#import "generated/frame/rain_blur"

// `ssfx_rain.ps` in two passes: the scene at an eighth of its size for the enhanced drops to bend, each texel the whole
// eight by eight block of the scene under it rather than one filtered read, so the copy holds still as the camera
// moves; then blurred as Screen Space Shaders blurs it, its middle four times and four taps two of its texels off on
// the diagonals, over eight. The sun shafts are in the scene already, where its `volumetric_buffer` adds them.

// How many of the scene's texels a texel of the copy spans across.
const SCALE: f32 = 8.0;

// The block of the scene a texel of the copy spans: sixteen filtered reads, each between four texels.
@fragment
fn fs_rain_block(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<f32> = 1.0 / vec2<f32>(textureDimensions(source));
  let at: vec2<f32> = in.clip.xy * SCALE * texel;
  var sum: vec3<f32> = vec3<f32>(0.0);

  for (var row: i32 = 0; row < 4; row++) {
    for (var column: i32 = 0; column < 4; column++) {
      let offset: vec2<f32> = (vec2<f32>(f32(column), f32(row)) * 2.0 - 3.0) * texel;

      sum += textureSampleLevel(source, source_sampler, at + offset, 0.0).rgb;
    }
  }

  return vec4<f32>(sum / 16.0, 1.0);
}

// A block of the copy, held within it.
fn block_at(texel: vec2<i32>) -> vec3<f32> {
  return textureLoad(source, clamp(texel, vec2<i32>(0), vec2<i32>(textureDimensions(source)) - 1), 0).rgb;
}

// The blocks blurred, each tap on a texel's middle.
@fragment
fn fs_rain_blur(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let at: vec2<i32> = vec2<i32>(in.clip.xy);
  var colour: vec3<f32> = block_at(at) * 4.0;

  colour += block_at(at + vec2<i32>(-2, 2)) + block_at(at + vec2<i32>(2, 2));
  colour += block_at(at + vec2<i32>(2, -2)) + block_at(at + vec2<i32>(-2, -2));

  return vec4<f32>(colour / 8.0, 1.0);
}
