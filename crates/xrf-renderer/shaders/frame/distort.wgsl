#import "common/camera"
#import "common/fullscreen"
#import "common/lighting"

// The engine's distortion (`combine_2`'s `USE_DISTORT`) laid on the frame at the drawn size before the temporal
// resolve, so the resolve smooths what it moves as it smooths the rest: the frame read where the distortion target moves
// each texel, `(distort.xy - .5) * def_distort`, then blurred towards the engine's bloom by the target's `z`,
// `lerp(img, bloom * def_hdr, distort.z)` (a hot haze, a wavy glass). After the resolve, read against the drawn depth
// and mask, a blade's border texels over water and a haze's edge flipped in and out each frame with the jitter and
// nothing smoothed them. A move that would read something standing nearer than what the texel shows reads the texel
// itself: a departure from the engine, whose water copies a railing standing in it into the water beside it.

// The frame before the distortion, what moves it, the G-buffer's depth, the engine's bloom and its sampler, and how far
// it moves and whether it blurs.
#import "generated/frame/distort"

// What the distortion target holds where nothing distorts it.
const NEUTRAL_DISTORTION: f32 = 127.0 / 255.0;

// Metres, and the share of its own depth, what a move reads may stand nearer than what the texel shows behind it.
const NEARER_MARGIN: f32 = 0.25;
const NEARER_SHARE: f32 = 0.02;

// How far along the view a texel lies, farther than anything where nothing was drawn.
fn view_distance(texel: vec2<i32>) -> f32 {
  let stored: f32 = textureLoad(depth_target, texel, 0);

  return select(-camera_view_position(vec2<f32>(texel) + 0.5, stored).z, 1e6, stored <= 0.0);
}

@fragment
fn fs_distort(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);
  let size: vec2<f32> = camera.viewport.xy;
  let distorted: vec4<f32> = textureLoad(distortion, texel, 0);
  let offset: vec2<f32> = (distorted.xy - NEUTRAL_DISTORTION) * distort.strength;
  var read: vec2<i32> = texel;

  // A texel nothing moves reads itself, with no depths to compare.
  if (any(offset != vec2<f32>(0.0))) {
    let moved: vec2<i32> = vec2<i32>(clamp(vec2<f32>(texel) + offset * size, vec2<f32>(0.0), size - 1.0));
    let here: f32 = view_distance(texel);

    if (view_distance(moved) >= here - max(NEARER_MARGIN, here * NEARER_SHARE)) {
      read = moved;
    }
  }

  let color: vec4<f32> = textureLoad(frame, read, 0);

  // The blur, where a bloom draws: towards the engine's, at its share of the screen.
  if (distort.is_blurred == 0u || distorted.z <= 0.0) {
    return color;
  }

  let bloom: vec3<f32> = textureSampleLevel(bloom_target, bloom_sampler, (vec2<f32>(texel) + 0.5) / size, 0.0).rgb;

  return vec4<f32>(mix(color.rgb, bloom * DEF_HDR, distorted.z), color.a);
}
