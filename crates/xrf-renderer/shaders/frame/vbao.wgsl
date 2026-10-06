#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/visibility_bitmask"

// VBAO, the visibility-bitmask ambient occlusion, at half the frame's size, a pixel per two by two of it: slices
// around the view searched into a visibility bitmask (`common/visibility_bitmask`), each sample an occluder `thickness`
// deep, so light passes behind grass, fences and branches; then accumulated over frames, carried by the motion target,
// and filtered. Visibility in red, from none to all, and the point's distance along the view in green; nothing drawn
// is all visible at no distance. The accumulation adds the frames it gathered in blue.

// Directions around the view, and steps each way along each: the quality's.
override SLICES: u32 = 2u;
override STEPS: u32 = 4u;

// The normals, depth and motion, the settings, the stage before's result and the last frame's accumulation.
#import "generated/frame/vbao"

const PI: f32 = 3.14159265;

// The share of the radius past which a sample is dropped by noise, more of them the further out.
const FALLOFF_START: f32 = 0.6;
// The steps' distances along a slice go as this power of an even spread, crowding towards the point.
const STEP_POWER: f32 = 1.6;
// Search pixels the radius spans over which the occlusion fades in; under one nothing is searched.
const FADE_PIXELS: f32 = 3.0;
// Share of a point's distance its last frame's distance may differ by and still be carried.
const HISTORY_TOLERANCE: f32 = 0.08;
// How far the carried visibility may stand outside what this frame's search found around the pixel.
const HISTORY_MARGIN: f32 = 0.15;
// Share of a point's distance a neighbour may stand off the plane through it and still be filtered with it.
const FILTER_TOLERANCE: f32 = 0.04;
// The golden ratio's fraction, which spreads the steps' noise.
const GOLDEN: f32 = 0.618034;

struct SearchPoint {
  depth: f32,
  position: vec3<f32>,
};

// The frame's point under a search pixel: the even texel of its two by two.
fn search_point(at: vec2<f32>, last: vec2<f32>) -> SearchPoint {
  let texel: vec2<f32> = clamp(at, vec2<f32>(0.0), last) * 2.0;
  let depth: f32 = textureLoad(depth_target, vec2<i32>(texel), 0);

  return SearchPoint(depth, camera_view_position(texel + 0.5, depth));
}

// Bayer's four by four ordered dither, from zero to fifteen sixteenths: each value once in every tile, neighbours far
// apart.
fn bayer(pixel: vec2<u32>) -> f32 {
  let y: u32 = pixel.y & 3u;
  let mixed: u32 = (pixel.x & 3u) ^ y;

  return f32(((mixed & 1u) << 3u) | ((y & 1u) << 2u) | (mixed & 2u) | ((y & 2u) >> 1u)) / 16.0;
}

@fragment
fn fs_search(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let frame: vec2<f32> = vec2<f32>(textureDimensions(depth_target));
  let last: vec2<f32> = floor((frame - 1.0) / 2.0);
  let pixel: vec2<f32> = floor(in.clip.xy);
  let center: SearchPoint = search_point(pixel, last);

  if (center.depth <= 0.0) {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
  }

  let position: vec3<f32> = center.position;
  let distance: f32 = -position.z;
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, vec2<i32>(pixel * 2.0), 0).xy);
  let to_camera: vec3<f32> = normalize(-position);
  let screen_radius: f32 = min(occlusion.radius / (distance * occlusion.spread), occlusion.reach);

  // A point whose radius is less than a pixel across has nothing to search.
  if (screen_radius < 1.0) {
    return vec4<f32>(1.0, distance, 0.0, 1.0);
  }

  let tile: vec2<u32> = vec2<u32>(pixel);
  let slice_noise: f32 = fract(bayer(tile) + 0.5 / 16.0 + occlusion.slice_noise);
  let step_noise: f32 = fract(bayer(tile.yx + vec2<u32>(1u, 2u)) + occlusion.step_noise);
  var visible: f32 = 0.0;
  var weights: f32 = 0.0;

  for (var slice: u32 = 0u; slice < SLICES; slice++) {
    let phi: f32 = (f32(slice) + slice_noise) / f32(SLICES) * PI;
    // Down the target is up the view.
    let along_screen: vec2<f32> = vec2<f32>(cos(phi), -sin(phi));
    let frame_slice: BitmaskSlice = bitmask_slice(normal, to_camera, vec3<f32>(cos(phi), sin(phi), 0.0));
    var mask: u32 = 0u;

    for (var step: u32 = 0u; step < STEPS; step++) {
      let jitter: f32 = fract(step_noise + f32(step + slice * STEPS) * GOLDEN);
      let reach: f32 = max(pow((f32(step) + jitter) / f32(STEPS), STEP_POWER) * screen_radius, 1.0);
      let offset: vec2<f32> = round(along_screen * reach);
      // Where past the falloff's start this step's samples are dropped.
      let falloff: f32 = occlusion.radius * mix(FALLOFF_START, 1.0, fract(jitter + 0.5 + slice_noise));

      for (var side: i32 = -1; side <= 1; side += 2) {
        let sample: SearchPoint = search_point(pixel + offset * f32(side), last);
        let front: vec3<f32> = sample.position - position;
        let reach_metres: f32 = length(front);

        // Nothing drawn there hides nothing, and nothing past the radius does.
        if (sample.depth <= 0.0 || reach_metres < 1e-4 || reach_metres > falloff) {
          continue;
        }

        let back: vec3<f32> = front + normalize(sample.position) * occlusion.thickness;

        mask |= bitmask_occluder(frame_slice, to_camera, front, back, f32(side));
      }
    }

    visible += frame_slice.weight * bitmask_visibility(mask);
    weights += frame_slice.weight;
  }

  let visibility: f32 = select(1.0, visible / weights, weights > 1e-4);

  return vec4<f32>(mix(1.0, visibility, saturate((screen_radius - 1.0) / FADE_PIXELS)), distance, 0.0, 1.0);
}

// This frame's search blended with the last frame's accumulation, carried to the pixel by the motion target and read
// bilinearly where it held the same surface: a running average over at most `frames` frames, kept near what the
// search finds around the pixel so a moved occluder's shadow does not trail.
@fragment
fn fs_accumulate(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let size: vec2<f32> = vec2<f32>(textureDimensions(source));
  let searched: vec2<f32> = textureLoad(source, vec2<i32>(pixel), 0).xy;
  let fresh: vec4<f32> = vec4<f32>(searched, 1.0, 1.0);

  if (searched.y <= 0.0) {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
  }

  if (occlusion.has_history < 0.5) {
    return fresh;
  }

  let frame: vec2<f32> = vec2<f32>(textureDimensions(depth_target));
  let texel: vec2<f32> = min(pixel * 2.0, frame - 1.0);
  let depth: f32 = textureLoad(depth_target, vec2<i32>(texel), 0);
  let uv: vec2<f32> = (texel + 0.5) / frame;
  let world: vec3<f32> = camera_unproject(vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0), depth);
  // How far along the last frame's view this point stood.
  let expected: f32 = (camera.motion_previous * vec4<f32>(world, 1.0)).w;
  let before: vec2<f32> = uv - textureLoad(motion_target, vec2<i32>(texel), 0).xy;

  if (any(before < vec2<f32>(0.0)) || any(before > vec2<f32>(1.0))) {
    return fresh;
  }

  let at: vec2<f32> = before * size - 0.5;
  let base: vec2<f32> = floor(at);
  let fraction: vec2<f32> = at - base;
  var carried: vec2<f32> = vec2<f32>(0.0);
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let held: vec4<f32> = textureLoad(history, vec2<i32>(clamp(base + offset, vec2<f32>(0.0), size - 1.0)), 0);
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let is_same: bool = held.y > 0.0 && abs(held.y - expected) <= expected * HISTORY_TOLERANCE;
    let weight: f32 = select(0.0, bilinear, is_same);

    carried += vec2<f32>(held.x, held.z) * weight;
    weights += weight;
  }

  if (weights < 1e-3) {
    return fresh;
  }

  // What the search found around the pixel, which the carried visibility is kept near.
  var low: f32 = searched.x;
  var high: f32 = searched.x;

  for (var index: i32 = 0; index < 9; index++) {
    let neighbour: vec2<f32> = textureLoad(source, vec2<i32>(clamp(pixel + vec2<f32>(f32(index % 3 - 1),
      f32(index / 3 - 1)), vec2<f32>(0.0), size - 1.0)), 0).xy;

    if (neighbour.y > 0.0) {
      low = min(low, neighbour.x);
      high = max(high, neighbour.x);
    }
  }

  let previous: f32 = clamp(carried.x / weights, low - HISTORY_MARGIN, high + HISTORY_MARGIN);
  let gathered: f32 = min(round(carried.y / weights) + 1.0, occlusion.frames);

  return vec4<f32>(mix(previous, searched.x, 1.0 / gathered), searched.y, gathered, 1.0);
}

// A neighbour's distance against the plane through the centre: its slope towards whichever neighbour lies nearer, so
// an edge does not tilt it, none where neither is drawn.
fn filter_slope(center: f32, before: f32, after: f32) -> f32 {
  let is_after: bool = after > 0.0 && (before <= 0.0 || abs(after - center) < abs(center - before));

  return select(select(0.0, center - before, before > 0.0), after - center, is_after);
}

// The accumulation, or the search alone, filtered every neighbour weighed by how near it lies to the plane through the
// centre: five by five with half-weight edges where few frames are gathered, which takes in each of the noise's four
// by four phases once, narrowing to three by three as the frames gather them instead; then the strength's curve.
@fragment
fn fs_filter(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let pixel: vec2<f32> = floor(in.clip.xy);
  let last: vec2<f32> = vec2<f32>(textureDimensions(source)) - 1.0;
  let gathered: vec3<f32> = textureLoad(source, vec2<i32>(pixel), 0).xyz;
  let center: vec2<f32> = gathered.xy;
  // How far the frames have gathered the noise's phases, from none (the search alone) to all.
  let focus: f32 = saturate(gathered.z / occlusion.frames);
  // The weights one and two pixels out, against one at the centre.
  let ring: vec2<f32> = vec2<f32>(mix(1.0, 0.75, focus), mix(0.5, 0.25, focus));

  if (center.y <= 0.0) {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
  }

  let left: f32 = textureLoad(source, vec2<i32>(max(pixel - vec2<f32>(1.0, 0.0), vec2<f32>(0.0))), 0).y;
  let right: f32 = textureLoad(source, vec2<i32>(min(pixel + vec2<f32>(1.0, 0.0), last)), 0).y;
  let up: f32 = textureLoad(source, vec2<i32>(max(pixel - vec2<f32>(0.0, 1.0), vec2<f32>(0.0))), 0).y;
  let down: f32 = textureLoad(source, vec2<i32>(min(pixel + vec2<f32>(0.0, 1.0), last)), 0).y;
  let slope: vec2<f32> = vec2<f32>(filter_slope(center.y, left, right), filter_slope(center.y, up, down));
  let tolerance: f32 = max(center.y * FILTER_TOLERANCE, 1e-3);
  var sum: f32 = 0.0;
  var weights: f32 = 0.0;

  for (var y: i32 = -2; y <= 2; y++) {
    for (var x: i32 = -2; x <= 2; x++) {
      let offset: vec2<f32> = vec2<f32>(f32(x), f32(y));
      let texel: vec2<f32> = textureLoad(source, vec2<i32>(clamp(pixel + offset, vec2<f32>(0.0), last)), 0).xy;
      let edges: vec2<f32> = select(select(vec2<f32>(1.0), vec2<f32>(ring.x), abs(offset) > vec2<f32>(0.5)),
        vec2<f32>(ring.y), abs(offset) > vec2<f32>(1.5));
      let error: f32 = abs(texel.y - (center.y + dot(slope, offset))) / tolerance;
      let weight: f32 = select(0.0, edges.x * edges.y * saturate(1.0 - error), texel.y > 0.0);

      sum += texel.x * weight;
      weights += weight;
    }
  }

  let visibility: f32 = clamp(sum / max(weights, 1e-4), 1e-4, 1.0);

  return vec4<f32>(pow(visibility, occlusion.power), center.y, 0.0, 1.0);
}
