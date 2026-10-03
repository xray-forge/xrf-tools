#import "common/camera"
#import "common/lighting"
#import "common/sun_shadow"
#import "common/fullscreen"

// `accum_volumetric_sun` and `combine_volumetric`: the light the air scatters towards the camera where the sun reaches
// it, stepped along each pixel's ray from what it shows back to the camera through the sun's shadow, the steps jittered
// a pixel apart; the sky as dense as the whole ray. Weighted towards the sun, in its colour, tonemapped and added to
// the frame.

@group(1) @binding(0) var depth_target: texture_depth_2d;
@group(1) @binding(1) var shadow_maps: texture_depth_2d_array;
@group(1) @binding(2) var<uniform> shadows: Shadows;
@group(1) @binding(3) var<uniform> lighting: Lighting;
@group(1) @binding(4) var<storage, read> exposure: Exposure;

// Closer than this along the view, metres, the air adds nothing: `depth > 0.3`.
const NEAR: f32 = 0.3;

// The engine's 64 texel jitter, stood in for by interleaved gradient noise: one value a pixel, from zero to one.
fn jitter(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

@fragment
fn fs_sun_shafts(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let density: f32 = lighting.shafts.x;
  let steps: f32 = lighting.shafts.y;

  if (density <= 0.0 || steps < 1.0) {
    return vec4<f32>(0.0);
  }

  let depth: f32 = textureLoad(depth_target, vec2<i32>(in.clip.xy), 0);
  var shown: f32 = 0.0;

  if (depth <= 0.0) {
    shown = density;
  } else {
    let point: vec3<f32> = camera_view_position(in.clip.xy, depth);
    let to_world: mat4x4<f32> = transpose(camera.view);
    // `(RAY_SAMPLES - J) / RAY_SAMPLES^2` of the way back each step.
    let step: vec3<f32> = point * ((steps - jitter(in.clip.xy)) / (steps * steps));
    let count: u32 = u32(steps);
    var at: vec3<f32> = point;

    for (var index: u32 = 0u; index < count; index++) {
      if (-at.z > NEAR) {
        let world: vec3<f32> = (to_world * vec4<f32>(at, 0.0)).xyz + camera.position.xyz;

        shown += density / steps * sun_shadow_tap(shadow_maps, shadows, world);
      }

      at -= step;
    }
  }

  // Facing the sun weighs one, facing away a fifth: `0.8 * (0.5 * cos + 0.5) + 0.2`.
  let facing: f32 = 0.8 * (0.5 * -lighting.to_sun.z + 0.5) + 0.2;
  let light: vec3<f32> = shown * facing * lighting.sun.rgb;

  return vec4<f32>(tonemap(light, frame_scale(lighting, exposure)), 0.0);
}
