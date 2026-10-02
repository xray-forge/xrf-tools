#import "common/camera"
#import "common/fullscreen"

// The viewport's finished scene put into its rectangle of the window.

@group(1) @binding(0) var scene: texture_2d<f32>;

// One step of the eight-bit window the frame is written to.
const OUTPUT_STEP: f32 = 1.0 / 255.0;

// Half a step either way by interleaved gradient noise (Jimenez, 2014), so a flat gradient falls between two steps as
// a fine grain rather than as bands.
fn output_dither(pixel: vec2<f32>) -> vec3<f32> {
  let noise: f32 = fract(fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))) * 52.9829189);

  return vec3<f32>((noise - 0.5) * OUTPUT_STEP);
}

@fragment
fn fs_present(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy - camera.viewport.zw);

  return vec4<f32>(textureLoad(scene, texel, 0).rgb + output_dither(in.clip.xy), 1.0);
}
