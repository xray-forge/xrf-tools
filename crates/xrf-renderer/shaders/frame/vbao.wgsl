#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/visibility_bitmask"

// The visibility-bitmask search at half the frame's size, a pixel per two by two of it: slices around the view searched
// into a visibility bitmask (`common/visibility_bitmask`), each sample an occluder `thickness` deep, so light passes
// behind grass, fences and branches; then accumulated over frames, carried by the motion target, and filtered. VBAO's
// occlusion is visibility in red, from none to all, and the point's distance along the view in green; nothing drawn is
// all visible at no distance. The accumulation adds the frames it gathered in blue.
//
// Lit, the same search gathers the indirect light: the light each sample's surface leaves (`fs_light_source`), through
// the bits it sets that were still clear, which are the share of the hemisphere it is seen through, by how far it faces
// the point. Colour, then the point's distance along the view once filtered.

// Directions around the view, and steps each way along each: the quality's; lit, steps each way past the occlusion's
// radius out to the light's.
override SLICES: u32 = 2u;
override STEPS: u32 = 4u;
override FAR_STEPS: u32 = 2u;

// The normals, depth and motion, the settings, the stage before's result and the last frame's accumulation; and, lit,
// the albedo, material and light, the light at the search's size, the stage before's light and the last frame's.
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
// How far the carried light may stand outside what this frame's search gathered around the pixel, a share of the
// brightest channel there.
const LIGHT_HISTORY_MARGIN: f32 = 0.25;
// Share of a point's distance a neighbour may stand off the plane through it and still be filtered with it.
const FILTER_TOLERANCE: f32 = 0.04;
// The golden ratio's fraction, which spreads the steps' noise.
const GOLDEN: f32 = 0.618034;
// The luminance a surface's light is held to, so a lamp's hot spot does not flare what it bounces onto.
const SOURCE_LIMIT: f32 = 4.0;
// How far a sample's surface must turn towards the point before all its light reaches it: a surface's light leaves
// it alike every way it faces, so only the back faces and those seen edge on are dropped.
const FACING_FADE: f32 = 0.2;
const LUMINANCE: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

struct SearchPoint {
  depth: f32,
  position: vec3<f32>,
  // The search pixel it lies under.
  pixel: vec2<f32>,
};

// The frame's point under a search pixel: the even texel of its two by two.
fn search_point(at: vec2<f32>, last: vec2<f32>) -> SearchPoint {
  let pixel: vec2<f32> = clamp(at, vec2<f32>(0.0), last);
  let texel: vec2<f32> = pixel * 2.0;
  let depth: f32 = textureLoad(depth_target, vec2<i32>(texel), 0);

  return SearchPoint(depth, camera_view_position(texel + 0.5, depth), pixel);
}

// Bayer's four by four ordered dither, from zero to fifteen sixteenths: each value once in every tile, neighbours far
// apart.
fn bayer(pixel: vec2<u32>) -> f32 {
  let y: u32 = pixel.y & 3u;
  let mixed: u32 = (pixel.x & 3u) ^ y;

  return f32(((mixed & 1u) << 3u) | ((y & 1u) << 2u) | (mixed & 2u) | ((y & 2u) >> 1u)) / 16.0;
}

// The light the frame's surfaces leave, at the search's size: the texel each search pixel is searched at, its albedo
// as the sun and the lights lit it. Self-lit surfaces leave none, their light already being the lamps', and the
// brightest are held to `SOURCE_LIMIT`.
@fragment
fn fs_light_source(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = min(vec2<i32>(floor(in.clip.xy) * 2.0), vec2<i32>(textureDimensions(depth_target)) - 1);

  if (textureLoad(depth_target, texel, 0) <= 0.0) {
    return vec4<f32>(0.0);
  }

  if (has_mark(textureLoad(material_target, texel, 0).a, MARK_EMISSIVE)) {
    return vec4<f32>(0.0);
  }

  let light: vec3<f32> = textureLoad(albedo_target, texel, 0).rgb * textureLoad(light_target, texel, 0).rgb;
  let luminance: f32 = dot(light, LUMINANCE);

  return vec4<f32>(light * min(1.0, SOURCE_LIMIT / max(luminance, 1e-4)), 1.0);
}

// What a search gives: the occlusion, and the light gathered where it is lit.
struct Searched {
  occlusion: vec4<f32>,
  light: vec4<f32>,
};

// The search at a pixel, gathering the light through each sample's newly set bits where `is_lit`.
fn search(clip: vec2<f32>, is_lit: bool) -> Searched {
  let frame: vec2<f32> = vec2<f32>(textureDimensions(depth_target));
  let last: vec2<f32> = floor((frame - 1.0) / 2.0);
  let pixel: vec2<f32> = floor(clip);
  let center: SearchPoint = search_point(pixel, last);
  let dark: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);

  if (center.depth <= 0.0) {
    return Searched(vec4<f32>(1.0, 0.0, 0.0, 1.0), dark);
  }

  let position: vec3<f32> = center.position;
  let distance: f32 = -position.z;
  let normal: vec3<f32> = octahedral_decode(textureLoad(normal_target, vec2<i32>(pixel * 2.0), 0).xy);
  let to_camera: vec3<f32> = normalize(-position);
  let screen_radius: f32 = min(occlusion.radius / (distance * occlusion.spread), occlusion.reach);

  // A point whose radius is less than a pixel across has nothing to search.
  if (screen_radius < 1.0) {
    return Searched(vec4<f32>(1.0, distance, 0.0, 1.0), dark);
  }

  let tile: vec2<u32> = vec2<u32>(pixel);
  let slice_noise: f32 = fract(bayer(tile) + 0.5 / 16.0 + occlusion.slice_noise);
  let step_noise: f32 = fract(bayer(tile.yx + vec2<u32>(1u, 2u)) + occlusion.step_noise);
  var visible: f32 = 0.0;
  var light: vec3<f32> = vec3<f32>(0.0);
  var weights: f32 = 0.0;

  // Lit, the light reaches further than the occlusion: steps past the occlusion's radius gather light alone, so the
  // occlusion's own samples are the same lit or not.
  let light_radius: f32 = min(occlusion.light_radius / (distance * occlusion.spread), occlusion.reach);
  let far_steps: u32 = select(0u, FAR_STEPS, is_lit && light_radius > screen_radius + 1.0);

  for (var slice: u32 = 0u; slice < SLICES; slice++) {
    let phi: f32 = (f32(slice) + slice_noise) / f32(SLICES) * PI;
    // Down the target is up the view.
    let along_screen: vec2<f32> = vec2<f32>(cos(phi), -sin(phi));
    let frame_slice: BitmaskSlice = bitmask_slice(normal, to_camera, vec3<f32>(cos(phi), sin(phi), 0.0));
    var mask: u32 = 0u;
    // What the light is seen through: every sample within the light's radius.
    var light_mask: u32 = 0u;
    var slice_light: vec3<f32> = vec3<f32>(0.0);

    for (var step: u32 = 0u; step < STEPS + far_steps; step++) {
      let jitter: f32 = fract(step_noise + f32(step + slice * STEPS) * GOLDEN);
      let is_near: bool = step < STEPS;
      let near: f32 = pow((f32(step) + jitter) / f32(STEPS), STEP_POWER) * screen_radius;
      let far: f32 = mix(screen_radius, light_radius, (f32(step - min(step, STEPS)) + jitter) / f32(max(far_steps, 1u)));
      let reach: f32 = max(select(far, near, is_near), 1.0);
      let offset: vec2<f32> = round(along_screen * reach);
      // Where past the falloff's start this step's samples are dropped.
      let spread: f32 = mix(FALLOFF_START, 1.0, fract(jitter + 0.5 + slice_noise));
      let falloff: f32 = select(0.0, occlusion.radius * spread, is_near);
      let light_falloff: f32 = select(falloff, occlusion.light_radius * spread, is_lit);

      for (var side: i32 = -1; side <= 1; side += 2) {
        let sample: SearchPoint = search_point(pixel + offset * f32(side), last);
        let front: vec3<f32> = sample.position - position;
        let reach_metres: f32 = length(front);

        // Nothing drawn there hides nothing, and nothing past the radius does.
        if (sample.depth <= 0.0 || reach_metres < 1e-4 || reach_metres > max(falloff, light_falloff)) {
          continue;
        }

        let back: vec3<f32> = front + normalize(sample.position) * occlusion.thickness;
        let bits: u32 = bitmask_occluder(frame_slice, to_camera, front, back, f32(side));

        if (reach_metres <= falloff) {
          mask |= bits;
        }

        if (!is_lit || reach_metres > light_falloff) {
          continue;
        }

        // The bits the sample sets that nothing nearer did: the share of the slice it is seen through.
        let fresh: u32 = bits & ~light_mask;

        light_mask |= bits;

        if (fresh != 0u) {
          let at: vec2<i32> = vec2<i32>(sample.pixel);
          let leaving: vec3<f32> = textureLoad(light_source, at, 0).rgb;
          let facing: f32 = dot(octahedral_decode(textureLoad(normal_target, at * 2, 0).xy), -front / reach_metres);

          slice_light += leaving * (f32(countOneBits(fresh)) * smoothstep(0.0, FACING_FADE, facing));
        }
      }
    }

    visible += frame_slice.weight * bitmask_visibility(mask);
    light += frame_slice.weight * slice_light / f32(BITMASK_BITS);
    weights += frame_slice.weight;
  }

  let has_weight: bool = weights > 1e-4;
  let visibility: f32 = select(1.0, visible / weights, has_weight);
  let fade: f32 = saturate((screen_radius - 1.0) / FADE_PIXELS);
  let gathered: vec3<f32> = select(vec3<f32>(0.0), light / weights, has_weight);

  return Searched(vec4<f32>(mix(1.0, visibility, fade), distance, 0.0, 1.0), vec4<f32>(gathered * fade, 1.0));
}

@fragment
fn fs_search(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return search(in.clip.xy, false).occlusion;
}

// The occlusion and the light gathered, into two targets.
struct SearchedTargets {
  @location(0) occlusion: vec4<f32>,
  @location(1) light: vec4<f32>,
};

@fragment
fn fs_search_lit(in: FullscreenVarying) -> SearchedTargets {
  let searched: Searched = search(in.clip.xy, true);

  return SearchedTargets(searched.occlusion, searched.light);
}

// This frame's search blended with the last frame's accumulation, carried to the pixel by the motion target and read
// bilinearly where it held the same surface: a running average over at most `frames` frames, kept near what the
// search finds around the pixel so a moved occluder's shadow, or a moved light's bounce, does not trail. The light's
// too where `is_lit`, by the same weights.
fn accumulate(clip: vec2<f32>, is_lit: bool) -> Searched {
  let pixel: vec2<f32> = floor(clip);
  let size: vec2<f32> = vec2<f32>(textureDimensions(source));
  let searched: vec2<f32> = textureLoad(source, vec2<i32>(pixel), 0).xy;
  let fresh: vec4<f32> = vec4<f32>(searched, 1.0, 1.0);
  var fresh_light: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0);

  if (is_lit) {
    fresh_light = vec4<f32>(textureLoad(gathered, vec2<i32>(pixel), 0).rgb, 1.0);
  }

  if (searched.y <= 0.0) {
    return Searched(vec4<f32>(1.0, 0.0, 0.0, 1.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
  }

  if (occlusion.has_history < 0.5) {
    return Searched(fresh, fresh_light);
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
    return Searched(fresh, fresh_light);
  }

  let at: vec2<f32> = before * size - 0.5;
  let base: vec2<f32> = floor(at);
  let fraction: vec2<f32> = at - base;
  var carried: vec2<f32> = vec2<f32>(0.0);
  var carried_light: vec3<f32> = vec3<f32>(0.0);
  var weights: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let offset: vec2<f32> = vec2<f32>(f32(corner & 1u), f32(corner >> 1u));
    let held_at: vec2<i32> = vec2<i32>(clamp(base + offset, vec2<f32>(0.0), size - 1.0));
    let held: vec4<f32> = textureLoad(history, held_at, 0);
    let bilinear: f32 = mix(1.0 - fraction.x, fraction.x, offset.x) * mix(1.0 - fraction.y, fraction.y, offset.y);
    let is_same: bool = held.y > 0.0 && abs(held.y - expected) <= expected * HISTORY_TOLERANCE;
    let weight: f32 = select(0.0, bilinear, is_same);

    carried += vec2<f32>(held.x, held.z) * weight;
    weights += weight;

    if (is_lit) {
      carried_light += textureLoad(light_history, held_at, 0).rgb * weight;
    }
  }

  if (weights < 1e-3) {
    return Searched(fresh, fresh_light);
  }

  // What the search found around the pixel, which the carried visibility and light are kept near.
  var low: f32 = searched.x;
  var high: f32 = searched.x;
  var low_light: vec3<f32> = fresh_light.rgb;
  var high_light: vec3<f32> = fresh_light.rgb;

  for (var index: i32 = 0; index < 9; index++) {
    let neighbour_at: vec2<i32> = vec2<i32>(clamp(pixel + vec2<f32>(f32(index % 3 - 1), f32(index / 3 - 1)),
      vec2<f32>(0.0), size - 1.0));
    let neighbour: vec2<f32> = textureLoad(source, neighbour_at, 0).xy;

    if (neighbour.y > 0.0) {
      low = min(low, neighbour.x);
      high = max(high, neighbour.x);

      if (is_lit) {
        let neighbour_light: vec3<f32> = textureLoad(gathered, neighbour_at, 0).rgb;

        low_light = min(low_light, neighbour_light);
        high_light = max(high_light, neighbour_light);
      }
    }
  }

  let previous: f32 = clamp(carried.x / weights, low - HISTORY_MARGIN, high + HISTORY_MARGIN);
  let frames: f32 = min(round(carried.y / weights) + 1.0, occlusion.frames);
  let accumulated: vec4<f32> = vec4<f32>(mix(previous, searched.x, 1.0 / frames), searched.y, frames, 1.0);

  if (!is_lit || occlusion.has_light_history < 0.5) {
    return Searched(accumulated, fresh_light);
  }

  let margin: f32 = max(max(high_light.r, high_light.g), high_light.b) * LIGHT_HISTORY_MARGIN;
  let previous_light: vec3<f32> = clamp(carried_light / weights, max(low_light - margin, vec3<f32>(0.0)),
    high_light + margin);

  return Searched(accumulated, vec4<f32>(mix(previous_light, fresh_light.rgb, 1.0 / frames), 1.0));
}

@fragment
fn fs_accumulate(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return accumulate(in.clip.xy, false).occlusion;
}

@fragment
fn fs_accumulate_lit(in: FullscreenVarying) -> SearchedTargets {
  let accumulated: Searched = accumulate(in.clip.xy, true);

  return SearchedTargets(accumulated.occlusion, accumulated.light);
}

// A neighbour's distance against the plane through the centre: its slope towards whichever neighbour lies nearer, so
// an edge does not tilt it, none where neither is drawn.
fn filter_slope(center: f32, before: f32, after: f32) -> f32 {
  let is_after: bool = after > 0.0 && (before <= 0.0 || abs(after - center) < abs(center - before));

  return select(select(0.0, center - before, before > 0.0), after - center, is_after);
}

// The accumulation, or the search alone, filtered every neighbour weighed by how near it lies to the plane through the
// centre: five by five with half-weight edges where few frames are gathered, which takes in each of the noise's four
// by four phases once, narrowing to three by three as the frames gather them instead; then the strength's curve. The
// light by the same weights where `is_lit`, times the intensity, its distance beside it.
fn filter_search(clip: vec2<f32>, is_lit: bool) -> Searched {
  let pixel: vec2<f32> = floor(clip);
  let last: vec2<f32> = vec2<f32>(textureDimensions(source)) - 1.0;
  let gathered_frames: vec3<f32> = textureLoad(source, vec2<i32>(pixel), 0).xyz;
  let center: vec2<f32> = gathered_frames.xy;
  // How far the frames have gathered the noise's phases, from none (the search alone) to all.
  let focus: f32 = saturate(gathered_frames.z / occlusion.frames);
  // The weights one and two pixels out, against one at the centre.
  let ring: vec2<f32> = vec2<f32>(mix(1.0, 0.75, focus), mix(0.5, 0.25, focus));

  if (center.y <= 0.0) {
    return Searched(vec4<f32>(1.0, 0.0, 0.0, 1.0), vec4<f32>(0.0));
  }

  let left: f32 = textureLoad(source, vec2<i32>(max(pixel - vec2<f32>(1.0, 0.0), vec2<f32>(0.0))), 0).y;
  let right: f32 = textureLoad(source, vec2<i32>(min(pixel + vec2<f32>(1.0, 0.0), last)), 0).y;
  let up: f32 = textureLoad(source, vec2<i32>(max(pixel - vec2<f32>(0.0, 1.0), vec2<f32>(0.0))), 0).y;
  let down: f32 = textureLoad(source, vec2<i32>(min(pixel + vec2<f32>(0.0, 1.0), last)), 0).y;
  let slope: vec2<f32> = vec2<f32>(filter_slope(center.y, left, right), filter_slope(center.y, up, down));
  let tolerance: f32 = max(center.y * FILTER_TOLERANCE, 1e-3);
  var sum: f32 = 0.0;
  var light: vec3<f32> = vec3<f32>(0.0);
  var weights: f32 = 0.0;

  for (var y: i32 = -2; y <= 2; y++) {
    for (var x: i32 = -2; x <= 2; x++) {
      let offset: vec2<f32> = vec2<f32>(f32(x), f32(y));
      let at: vec2<i32> = vec2<i32>(clamp(pixel + offset, vec2<f32>(0.0), last));
      let texel: vec2<f32> = textureLoad(source, at, 0).xy;
      let edges: vec2<f32> = select(select(vec2<f32>(1.0), vec2<f32>(ring.x), abs(offset) > vec2<f32>(0.5)),
        vec2<f32>(ring.y), abs(offset) > vec2<f32>(1.5));
      let error: f32 = abs(texel.y - (center.y + dot(slope, offset))) / tolerance;
      let weight: f32 = select(0.0, edges.x * edges.y * saturate(1.0 - error), texel.y > 0.0);

      sum += texel.x * weight;
      weights += weight;

      if (is_lit) {
        light += textureLoad(gathered, at, 0).rgb * weight;
      }
    }
  }

  let visibility: f32 = clamp(sum / max(weights, 1e-4), 1e-4, 1.0);

  return Searched(
    vec4<f32>(pow(visibility, occlusion.power), center.y, 0.0, 1.0),
    vec4<f32>(light / max(weights, 1e-4) * occlusion.intensity, center.y),
  );
}

@fragment
fn fs_filter(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return filter_search(in.clip.xy, false).occlusion;
}

@fragment
fn fs_filter_lit(in: FullscreenVarying) -> SearchedTargets {
  let filtered: Searched = filter_search(in.clip.xy, true);

  return SearchedTargets(filtered.occlusion, filtered.light);
}

@fragment
fn fs_filter_light(in: FullscreenVarying) -> @location(0) vec4<f32> {
  return filter_search(in.clip.xy, true).light;
}
