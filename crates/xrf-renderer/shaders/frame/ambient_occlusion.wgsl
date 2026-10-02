#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"

// Ambient occlusion from the frame's depth: XeGTAO (MIT) at half the frame's size, a pixel per two by two of it, then
// denoised one way and the other. Visibility in red, from none to all, and the point's distance along the view in
// green; nothing drawn is all visible at no distance.

struct AmbientOcclusion {
  // Metres around a point that what stands there occludes it from: `EffectRadius * RadiusMultiplier`.
  radius: f32,
  // What the visibility is raised to: XeGTAO's curve times the strength.
  power: f32,
  // Metres one pixel of the search target spans at a metre from the camera.
  spread: f32,
  // Pixels of the search target a horizon is searched across at most.
  reach: f32,
};

// Directions around the view, and steps each way along each: the quality's.
override SLICES: u32 = 3u;
override STEPS: u32 = 3u;

@group(1) @binding(0) var normal_target: texture_2d<f32>;
@group(1) @binding(1) var depth_target: texture_depth_2d;
@group(1) @binding(2) var<uniform> occlusion: AmbientOcclusion;
// What the other way of the denoise reads.
@group(1) @binding(3) var source: texture_2d<f32>;

const PI: f32 = 3.14159265;

// `EffectFalloffRange`: the outer share of the radius over which an occluder fades out.
const FALLOFF_RANGE: f32 = 0.615;
// `SampleDistributionPower`: steps crowd towards the point, where occluders matter most.
const DISTRIBUTION_POWER: f32 = 2.0;
// `PixelTooCloseThreshold`: the first step lands this many pixels out, off the point's own texel.
const TOO_CLOSE: f32 = 1.3;
// Share of a point's distance that a neighbour's may differ by and still be denoised with it.
const DENOISE_TOLERANCE: f32 = 0.02;

struct OcclusionPoint {
  depth: f32,
  position: vec3<f32>,
};

// The frame's point under a search pixel: the even texel of its two by two.
fn to_point(at: vec2<f32>, last: vec2<f32>) -> OcclusionPoint {
  let texel: vec2<f32> = clamp(at, vec2<f32>(0.0), last) * 2.0;
  let depth: f32 = textureLoad(depth_target, vec2<i32>(texel), 0);

  return OcclusionPoint(depth, camera_view_position(texel + 0.5, depth));
}

// A four by four ordered dither, each of sixteen values once in every tile.
fn tile_noise(x: f32, y: f32) -> f32 {
  let low: f32 = abs((x % 2.0) - (y % 2.0)) * 2.0 + (y % 2.0);
  let hx: f32 = floor(x / 2.0) % 2.0;
  let hy: f32 = floor(y / 2.0) % 2.0;

  return low * 4.0 + abs(hx - hy) * 2.0 + hy;
}

// How far one sample raises a horizon: up to it by how near it stands, one past the radius not at all.
fn horizon_cos(sample: OcclusionPoint, origin: vec3<f32>, to_camera: vec3<f32>, falloff: vec2<f32>, low: f32,
  horizon: f32) -> f32 {
  let delta: vec3<f32> = sample.position - origin;
  let distance: f32 = max(length(delta), 1e-4);
  let weight: f32 = saturate(distance * falloff.x + falloff.y);
  let cosine: f32 = mix(low, dot(delta / distance, to_camera), weight);

  // Nothing drawn there hides nothing.
  return max(horizon, select(low, cosine, sample.depth > 0.0));
}

// `XeGTAO_MainPass`: the horizons each way along a few directions around the view, and the cosine weighted visibility
// between them. The noise turning the directions is tiled four by four and still from frame to frame, so the denoise
// takes it out and a still view stays still.
@fragment
fn fs_search(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let frame: vec2<f32> = vec2<f32>(textureDimensions(depth_target));
  let last: vec2<f32> = floor((frame - 1.0) / 2.0);
  let pixel: vec2<f32> = floor(in.clip.xy);
  let center: OcclusionPoint = to_point(pixel, last);

  if (center.depth <= 0.0) {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
  }

  // Towards the camera a little, as XeGTAO moves it, against the depth's own imprecision.
  let position: vec3<f32> = center.position * 0.99999;
  let distance: f32 = -position.z;
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, vec2<i32>(pixel * 2.0), 0).xy);
  let to_camera: vec3<f32> = normalize(-position);
  let falloff_range: f32 = occlusion.radius * FALLOFF_RANGE;
  let falloff: vec2<f32> = vec2<f32>(-1.0 / falloff_range, occlusion.radius * (1.0 - FALLOFF_RANGE) / falloff_range + 1.0);
  let screen_radius: f32 = min(occlusion.radius / (distance * occlusion.spread), occlusion.reach);

  // A point whose radius is less than a pixel across has nothing to search.
  if (screen_radius < 1.0) {
    return vec4<f32>(1.0, distance, 0.0, 1.0);
  }

  let min_step: f32 = TOO_CLOSE / screen_radius;
  let slice_noise: f32 = (tile_noise(pixel.x, pixel.y) + 0.5) / 16.0;
  let step_noise: f32 = tile_noise(pixel.y, pixel.x) / 16.0;
  let half_pi: f32 = PI * 0.5;
  var visibility: f32 = saturate((10.0 - screen_radius) / 100.0) * 0.5;

  for (var slice: u32 = 0u; slice < SLICES; slice++) {
    let phi: f32 = (slice_noise + f32(slice)) / f32(SLICES) * PI;
    // Down the target is up the view.
    let omega: vec2<f32> = vec2<f32>(cos(phi), -sin(phi)) * screen_radius;
    let direction: vec3<f32> = vec3<f32>(cos(phi), sin(phi), 0.0);
    let ortho_direction: vec3<f32> = direction - to_camera * dot(direction, to_camera);
    let axis: vec3<f32> = normalize(cross(ortho_direction, to_camera));
    let projected_normal: vec3<f32> = normal - axis * dot(normal, axis);
    let projected_length: f32 = length(projected_normal);
    let cos_normal: f32 = saturate(dot(projected_normal, to_camera) / max(projected_length, 1e-4));
    let n: f32 = sign(dot(ortho_direction, projected_normal)) * acos(cos_normal);
    // The tangent plane each way: no horizon found lies below it.
    let low_cos0: f32 = cos(n + half_pi);
    let low_cos1: f32 = cos(n - half_pi);
    var horizon_cos0: f32 = low_cos0;
    var horizon_cos1: f32 = low_cos1;

    for (var step: u32 = 0u; step < STEPS; step++) {
      let noise: f32 = fract(step_noise + f32(slice + step * STEPS) * 0.6180339887);
      let s: f32 = pow((noise + f32(step)) / f32(STEPS), DISTRIBUTION_POWER) + min_step;
      let offset: vec2<f32> = round(omega * s);

      horizon_cos0 = horizon_cos(to_point(pixel + offset, last), position, to_camera, falloff, low_cos0, horizon_cos0);
      horizon_cos1 = horizon_cos(to_point(pixel - offset, last), position, to_camera, falloff, low_cos1, horizon_cos1);
    }

    let h0: f32 = n + clamp(-acos(clamp(horizon_cos1, -1.0, 1.0)) - n, -half_pi, half_pi);
    let h1: f32 = n + clamp(acos(clamp(horizon_cos0, -1.0, 1.0)) - n, -half_pi, half_pi);
    let arc0: f32 = (cos_normal + 2.0 * h0 * sin(n) - cos(2.0 * h0 - n)) / 4.0;
    let arc1: f32 = (cos_normal + 2.0 * h1 * sin(n) - cos(2.0 * h1 - n)) / 4.0;

    visibility += mix(projected_length, 1.0, 0.05) * (arc0 + arc1);
  }

  return vec4<f32>(max(pow(max(visibility / f32(SLICES), 1e-4), occlusion.power), 0.03), distance, 0.0, 1.0);
}

// One way of XeGTAO's denoise: the visibility averaged four pixels wide, each weighed by how near its distance lies
// to the one a surface through the centre would have there, so an edge keeps its side.
fn denoise(pixel: vec2<f32>, step: vec2<f32>) -> vec4<f32> {
  let last: vec2<f32> = vec2<f32>(textureDimensions(source)) - 1.0;
  let center: vec4<f32> = textureLoad(source, vec2<i32>(pixel), 0);
  let distance: f32 = center.y;
  // The slope through the centre, from its nearest neighbours, so a floor seen edge on is not an edge.
  let slope: f32 = (textureLoad(source, vec2<i32>(clamp(pixel + step, vec2<f32>(0.0), last)), 0).y
    - textureLoad(source, vec2<i32>(clamp(pixel - step, vec2<f32>(0.0), last)), 0).y) * 0.5;
  let tolerance: f32 = max(distance * DENOISE_TOLERANCE, 1e-3);
  // Four pixels wide, the ends at half weight: every phase of the noise's four pixel tile counts once.
  let offsets: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-2.0, 0.5),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(2.0, 0.5)
  );
  var sum: f32 = center.x;
  var weights: f32 = 1.0;

  for (var index: u32 = 0u; index < 4u; index++) {
    let offset: f32 = offsets[index].x;
    let texel: vec4<f32> = textureLoad(source, vec2<i32>(clamp(pixel + step * offset, vec2<f32>(0.0), last)), 0);
    let delta: f32 = texel.y - distance;
    let error: f32 = min(abs(delta), abs(delta - slope * offset));
    let weight: f32 = saturate(1.25 - error / tolerance) * offsets[index].y;

    sum += texel.x * weight;
    weights += weight;
  }

  return vec4<f32>(sum / weights, distance, 0.0, 1.0);
}

@fragment
fn fs_denoise_x(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return denoise(floor(in.clip.xy), vec2<f32>(1.0, 0.0));
}

@fragment
fn fs_denoise_y(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return denoise(floor(in.clip.xy), vec2<f32>(0.0, 1.0));
}
