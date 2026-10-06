#import "common/camera"
#import "common/lighting"
#import "common/sun_shadow"
#import "common/fullscreen"

// `accum_volumetric_sun` and `combine_volumetric`: the light the air scatters towards the camera where the sun reaches
// it, stepped along each ray from what it shows back to the camera through the sun's shadow, the steps jittered a pixel
// apart, at half the frame's size and brought up by depth; the sky as dense as the whole ray. Weighted towards the sun,
// in its colour, tonemapped and added to the frame.

#import "generated/frame/sun_shafts"

// Closer than this along the view, metres, the air adds nothing: `depth > 0.3`.
const NEAR: f32 = 0.3;

// The engine's 64 texel jitter, stood in for by interleaved gradient noise: one value a pixel, from zero to one.
fn jitter(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

// The light added to the scene, tonemapped, and to its high part, as `combine_volumetric.ps` adds to both.
struct ShaftsOutput {
  @location(0) low: vec4<f32>,
  @location(1) high: vec4<f32>,
};

// How much of the light the air scatters a ray from the camera to what a frame's texel shows gathers: its steps back
// through the sun's shadow, `(RAY_SAMPLES - J) / RAY_SAMPLES^2` of the way each, jittered by `noise`.
fn shafts_gathered(texel: vec2<i32>, depth: f32, density: f32, steps: f32, noise: f32) -> f32 {
  let point: vec3<f32> = camera_view_position(vec2<f32>(texel) + 0.5, depth);
  let to_world: mat4x4<f32> = transpose(camera.view);
  let step: vec3<f32> = point * ((steps - noise) / (steps * steps));
  let count: u32 = u32(steps);
  var at: vec3<f32> = point;
  var shown: f32 = 0.0;

  for (var index: u32 = 0u; index < count; index++) {
    if (-at.z > NEAR) {
      let world: vec3<f32> = (to_world * vec4<f32>(at, 0.0)).xyz + camera.position.xyz;

      shown += density / steps * sun_shadow_tap(shadow_maps, shadows, world);
    }

    at -= step;
  }

  return shown;
}

// The frame's texel a half-size texel of the march stands for: the first of its two by two.
fn marched_texel(pixel: vec2<i32>) -> vec2<i32> {
  return min(pixel * 2, vec2<i32>(textureDimensions(depth_target)) - 1);
}

// The march at half the frame's size, the light shafts being soft: what each texel's frame texel gathers, the sky
// left to the composite, which knows it as dense as the whole ray.
@fragment
fn fs_sun_shafts_march(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let density: f32 = lighting.shafts.x;
  let steps: f32 = lighting.shafts.y;
  let texel: vec2<i32> = marched_texel(vec2<i32>(in.clip.xy));
  let depth: f32 = textureLoad(depth_target, texel, 0);

  if (density <= 0.0 || steps < 1.0 || depth <= 0.0) {
    return vec4<f32>(0.0);
  }

  return vec4<f32>(shafts_gathered(texel, depth, density, steps, jitter(in.clip.xy)), 0.0, 0.0, 0.0);
}

// The march brought up to a frame pixel: the four half-size texels about it by their bilinear share and how near the
// distance each one's frame texel shows lies to the pixel's, so the shafts keep to the edges the depth draws.
fn shafts_upsampled(pixel: vec2<f32>, distance: f32) -> f32 {
  let last: vec2<i32> = vec2<i32>(textureDimensions(shafts)) - 1;
  let at: vec2<f32> = pixel * 0.5 - 0.25;
  let base: vec2<f32> = floor(at);
  let fraction: vec2<f32> = at - base;
  var sum: f32 = 0.0;
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let half_texel: vec2<i32> = clamp(vec2<i32>(base + offset), vec2<i32>(0), last);
    let texel: vec2<i32> = marched_texel(half_texel);
    let stored: f32 = textureLoad(depth_target, texel, 0);
    let there: f32 = select(1e6, -camera_view_position(vec2<f32>(texel) + 0.5, max(stored, 1e-7)).z, stored > 0.0);
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let difference: f32 = abs(there - distance) / max(distance, 1e-3);
    let weight: f32 = (bilinear + 1e-4) / (difference * difference + 1e-3);

    sum += textureLoad(shafts, half_texel, 0).r * weight;
    weights += weight;
  }

  return sum / max(weights, 1e-6);
}

@fragment
fn fs_sun_shafts(in: FullscreenVarying) -> ShaftsOutput {
  let density: f32 = lighting.shafts.x;
  let steps: f32 = lighting.shafts.y;

  if (density <= 0.0 || steps < 1.0) {
    return ShaftsOutput(vec4<f32>(0.0), vec4<f32>(0.0));
  }

  let depth: f32 = textureLoad(depth_target, vec2<i32>(in.clip.xy), 0);
  var shown: f32 = density;

  if (depth > 0.0) {
    shown = shafts_upsampled(floor(in.clip.xy), -camera_view_position(in.clip.xy, depth).z);
  }

  // Facing the sun weighs one, facing away a fifth: `0.8 * (0.5 * cos + 0.5) + 0.2`.
  let facing: f32 = 0.8 * (0.5 * -lighting.to_sun.z + 0.5) + 0.2;
  let light: vec3<f32> = shown * facing * lighting.sun.rgb;

  let scale: f32 = frame_scale(lighting, exposure);

  return ShaftsOutput(vec4<f32>(tonemap(light, scale), 0.0), vec4<f32>(tonemap_high(light, scale), 0.0));
}
