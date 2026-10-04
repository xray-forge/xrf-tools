#import "common/camera"
#import "common/flares"
#import "common/sun_shadow"

// How much of the sun shows, as `CLensFlare::OnFrame` measures it by five rays from the camera, eased towards at
// `BLEND_DEC_SPEED` a second. A ray the screen holds is tested against the frame's depth, as the sky shows only where
// nothing was drawn; one off the screen against the sun's shadow at the camera.

@group(1) @binding(0) var<uniform> flares: Flares;
// x: how much of the sun shows, eased, `fBlend`.
@group(1) @binding(1) var<storage, read_write> state: array<f32, 4>;
@group(1) @binding(2) var depth_target: texture_depth_2d;
@group(1) @binding(3) var shadow_maps: texture_depth_2d_array;
@group(1) @binding(4) var<uniform> shadows: Shadows;

// The rays' bends across and up, `RayDeltas`.
const RAYS: array<vec2<f32>, 5> = array<vec2<f32>, 5>(
  vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, -1.0), vec2<f32>(0.0, 1.0),
);

// Whether a view space ray reaches the sky: unhidden on the screen, or lit by the sun where it leaves the screen.
fn ray_visibility(toward: vec3<f32>) -> f32 {
  let clip: vec4<f32> = camera.projection * vec4<f32>(toward, 1.0);
  let ndc: vec2<f32> = clip.xy / clip.w;

  if (clip.w > 0.0 && all(abs(ndc) < vec2<f32>(1.0))) {
    let size: vec2<f32> = camera.viewport.xy;
    let texel: vec2<i32> = vec2<i32>(vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * size);
    let depth: f32 = textureLoad(depth_target, clamp(texel, vec2<i32>(0), vec2<i32>(size) - 1), 0);

    return select(0.0, 1.0, depth <= 0.0);
  }

  return sun_shadow(shadow_maps, shadows, camera.position.xyz, flares.sun.xyz, 1.0);
}

@compute @workgroup_size(1)
fn cs_visibility() {
  let to_sun: vec3<f32> = flares.to_sun.xyz;
  var shown: f32 = 0.0;

  if (-to_sun.z > 0.01) {
    for (var ray: u32 = 0u; ray < 5u; ray++) {
      let bend: vec2<f32> = RAYS[ray] * FLARE_RAY_SPREAD;

      shown += ray_visibility(normalize(to_sun + vec3<f32>(bend.x, -bend.y, 0.0)));
    }

    shown /= 5.0;
  }

  let eased: f32 = state[0];
  let step: f32 = FLARE_BLEND_SPEED * flares.to_sun.w;

  state[0] = clamp(eased + clamp(shown - eased, -step, step), 0.0, 1.0);
}
