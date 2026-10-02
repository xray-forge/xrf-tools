#import "common/camera"
#import "common/fullscreen"

// The viewport's finished scene put into its rectangle of the window, moved where the water distorts it.

@group(1) @binding(0) var scene: texture_2d<f32>;
// How far the water moves what is seen through it, around what the target is cleared to.
@group(1) @binding(1) var distortion: texture_2d<f32>;
@group(1) @binding(2) var depth_target: texture_depth_2d;

// One step of the eight-bit window the frame is written to.
const OUTPUT_STEP: f32 = 1.0 / 255.0;

// What the distortion target holds where nothing distorts it.
const NEUTRAL_DISTORTION: f32 = 127.0 / 255.0;

// Metres, and the share of its own depth, what a move reads may stand nearer than what the pixel shows behind it.
const NEARER_MARGIN: f32 = 0.25;
const NEARER_SHARE: f32 = 0.02;

// Half a step either way by interleaved gradient noise (Jimenez, 2014), so a flat gradient falls between two steps as
// a fine grain rather than as bands.
fn output_dither(pixel: vec2<f32>) -> vec3<f32> {
  let noise: f32 = fract(fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))) * 52.9829189);

  return vec3<f32>((noise - 0.5) * OUTPUT_STEP);
}

// How far along the view a texel of the viewport's targets lies, farther than anything where nothing was drawn.
fn view_distance(texel: vec2<i32>) -> f32 {
  let stored: f32 = textureLoad(depth_target, texel, 0);

  return select(-camera_view_position(vec2<f32>(texel) + 0.5, stored).z, 1e6, stored <= 0.0);
}

// `combine_2`'s `USE_DISTORT`: the scene read where the distortion target moves each pixel, `(distort.xy - .5) *
// def_distort`. A move that would read something standing nearer than what the pixel shows reads the pixel itself: a
// departure from the engine, whose water copies a railing standing in it into the water beside it.
@fragment
fn fs_present(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = camera.viewport.xy;
  let texel: vec2<i32> = vec2<i32>(in.clip.xy - camera.viewport.zw);
  var read: vec2<i32> = texel;
  let strength: f32 = camera.switches.w;

  if (strength > 0.0) {
    let offset: vec2<f32> = (textureLoad(distortion, texel, 0).xy - NEUTRAL_DISTORTION) * strength;
    let moved: vec2<i32> = vec2<i32>(clamp(vec2<f32>(texel) + offset * size, vec2<f32>(0.0), size - 1.0));
    let here: f32 = view_distance(texel);
    let there: f32 = view_distance(moved);

    if (there >= here - max(NEARER_MARGIN, here * NEARER_SHARE)) {
      read = moved;
    }
  }

  return vec4<f32>(textureLoad(scene, read, 0).rgb + output_dither(in.clip.xy), 1.0);
}
