#import "common/fullscreen"
#import "frame/fsr/common"

// `ffx_fsr2_autogen_reactive_pass.hlsl`: the reactive mask from what the blended surfaces changed, with its tonemap,
// component-maximum and threshold flags on.

@group(0) @binding(1) var opaque: texture_2d<f32>;
@group(0) @binding(2) var color: texture_2d<f32>;

// A change of a channel under which a texel is not reactive.
const REACTIVE_THRESHOLD: f32 = 0.2;
// How reactive a texel past the threshold counts.
const REACTIVE_VALUE: f32 = 0.9;

// `Tonemap`.
fn tonemapped(rgb: vec3<f32>) -> vec3<f32> {
  return rgb / (max(max(rgb.x, max(rgb.y, rgb.z)), 0.0) + 1.0);
}

@fragment
fn fs_reactive(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = clamped_texel(floor(in.clip.xy), fsr.render_size);
  let before: vec3<f32> = tonemapped(textureLoad(opaque, texel, 0).rgb);
  let after: vec3<f32> = tonemapped(textureLoad(color, texel, 0).rgb);
  let delta: vec3<f32> = abs(after - before);
  let reactive: f32 = select(REACTIVE_VALUE, 0.0, max(delta.x, max(delta.y, delta.z)) < REACTIVE_THRESHOLD);

  return vec4<f32>(reactive, 0.0, 0.0, 1.0);
}
