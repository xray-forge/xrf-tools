#import "common/lighting"
#import "common/sky"
#import "common/fullscreen"

// The sky as the frame draws it, both skies and the clouds over them, blurred into the haze map the distance fades
// into: each texel's direction, the bearing across and the height up, averaged over a flat ellipse about it.

@group(1) @binding(0) var<uniform> lighting: Lighting;
@group(1) @binding(1) var<storage, read> exposure: Exposure;

// Texels across, one a bearing, and down, one a height, as `frame/view_targets.rs` sizes the map.
const HAZE_SIZE: vec2<f32> = vec2<f32>(64.0, 32.0);

// Taps a ring holds, and the rings: a centre and rings of eight around it.
const RING_TAPS: i32 = 8;
const RINGS: array<f32, 3> = array<f32, 3>(0.33, 0.66, 1.0);

// How far the blur reaches either way, half its angle: across, wide enough that no cloud's shape survives; up and
// down, little, so the sky's own gradient stays and the horizon never reads the darker rim under it.
const ACROSS: f32 = 15.0 * SKY_PI / 180.0;
const UP: f32 = 3.0 * SKY_PI / 180.0;

// The sky along a direction as the frame draws it, lifted above the fold: the box's rim under it is what the sky
// draws below the horizon, and no haze of it.
fn sample_sky(direction: vec3<f32>, scale: f32) -> vec3<f32> {
  let lifted: vec3<f32> = normalize(vec3<f32>(direction.x, max(direction.y, length(direction.xz) * HAZE_TOP), direction.z));

  return sky_shown(lighting, lifted, scale);
}

@fragment
fn fs_sky_haze(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let uv: vec2<f32> = in.clip.xy / HAZE_SIZE;
  let scale: f32 = frame_scale(lighting, exposure);
  let bearing: f32 = (uv.x - 0.5) * SKY_PI * 2.0;
  // As `sky_haze_coordinates` reads it back: a target sampled where it was drawn, the height from the nadir up.
  let height: f32 = (uv.y - 0.5) * SKY_PI;
  let direction: vec3<f32> = vec3<f32>(cos(height) * sin(bearing), sin(height), cos(height) * cos(bearing));
  // A frame about it, whatever its height: the vertical never lies along it but at the poles, where any frame will do.
  let side: vec3<f32> = normalize(vec3<f32>(direction.z, 0.0, -direction.x) + vec3<f32>(1e-4, 0.0, 0.0));
  let up: vec3<f32> = normalize(cross(direction, side));
  var sum: vec3<f32> = sample_sky(direction, scale);
  var taps: f32 = 1.0;
  var rings: array<f32, 3> = RINGS;

  for (var ring: i32 = 0; ring < 3; ring++) {
    let reach: f32 = rings[ring];
    let across: f32 = tan(ACROSS * reach);
    let upward: f32 = tan(UP * reach);

    for (var tap: i32 = 0; tap < RING_TAPS; tap++) {
      let angle: f32 = (f32(tap) + reach) / f32(RING_TAPS) * SKY_PI * 2.0;
      let turned: vec3<f32> = normalize(direction + side * cos(angle) * across + up * sin(angle) * upward);

      sum += sample_sky(turned, scale);
      taps += 1.0;
    }
  }

  return vec4<f32>(sum / taps, 1.0);
}
