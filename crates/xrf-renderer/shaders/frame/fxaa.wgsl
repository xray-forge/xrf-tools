#import "common/fullscreen"

// FXAA 3.11 (Timothy Lottes, NVIDIA), the PC quality preset three's `FXAANode` draws: each edge found by the contrast
// of the luma around a pixel, walked along to its ends, and the pixel blended across it by where along the edge it
// lies, with a sub-pixel term for features thinner than a pixel.

@group(0) @binding(0) var frame: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

// The least contrast an edge needs, and the least against the brightest of its neighbours.
const EDGE_THRESHOLD: f32 = 0.166;
const EDGE_THRESHOLD_MIN: f32 = 0.0833;
// How much of the sub-pixel term is kept.
const SUBPIX: f32 = 0.75;
// Steps along an edge searched each way, and how far each one goes.
const SEARCH_STEPS: u32 = 12u;
var<private> STEPS: array<f32, 12> = array<f32, 12>(1.0, 1.0, 1.0, 1.0, 1.0, 1.5, 2.0, 2.0, 2.0, 2.0, 4.0, 8.0);

fn luma(color: vec3<f32>) -> f32 {
  return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

fn luma_at(uv: vec2<f32>) -> f32 {
  return luma(textureSampleLevel(frame, frame_sampler, uv, 0.0).rgb);
}

@fragment
fn fs_fxaa(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(frame));
  let texel: vec2<f32> = 1.0 / size;
  let uv: vec2<f32> = in.clip.xy * texel;
  let center: vec4<f32> = textureSampleLevel(frame, frame_sampler, uv, 0.0);
  let m: f32 = luma(center.rgb);
  let n: f32 = luma_at(uv + vec2<f32>(0.0, -texel.y));
  let s: f32 = luma_at(uv + vec2<f32>(0.0, texel.y));
  let e: f32 = luma_at(uv + vec2<f32>(texel.x, 0.0));
  let w: f32 = luma_at(uv + vec2<f32>(-texel.x, 0.0));
  let most: f32 = max(max(n, s), max(max(e, w), m));
  let least: f32 = min(min(n, s), min(min(e, w), m));
  let range: f32 = most - least;

  if (range < max(EDGE_THRESHOLD_MIN, most * EDGE_THRESHOLD)) {
    return center;
  }

  let nw: f32 = luma_at(uv - texel);
  let se: f32 = luma_at(uv + texel);
  let ne: f32 = luma_at(uv + vec2<f32>(texel.x, -texel.y));
  let sw: f32 = luma_at(uv + vec2<f32>(-texel.x, texel.y));

  // The sub-pixel blend: how far the pixel stands from the mean of its eight neighbours against the range.
  let mean: f32 = (2.0 * (n + s + e + w) + nw + ne + sw + se) / 12.0;
  let subpix_a: f32 = saturate(abs(mean - m) / range);
  let subpix_b: f32 = (-2.0 * subpix_a + 3.0) * subpix_a * subpix_a;
  let subpix: f32 = subpix_b * subpix_b * SUBPIX;

  // Whether the edge runs across or down.
  let horizontal: f32 = abs(nw + sw - 2.0 * w) + 2.0 * abs(n + s - 2.0 * m) + abs(ne + se - 2.0 * e);
  let vertical: f32 = abs(nw + ne - 2.0 * n) + 2.0 * abs(w + e - 2.0 * m) + abs(sw + se - 2.0 * s);
  let is_horizontal: bool = horizontal >= vertical;

  // The side of the edge the pixel's neighbour across it is on, and the step across it.
  let before: f32 = select(w, n, is_horizontal);
  let after: f32 = select(e, s, is_horizontal);
  let gradient_before: f32 = abs(before - m);
  let gradient_after: f32 = abs(after - m);
  let is_before: bool = gradient_before >= gradient_after;
  let gradient: f32 = max(gradient_before, gradient_after) * 0.25;
  var step_length: f32 = select(texel.x, texel.y, is_horizontal);
  var local_mean: f32 = 0.5 * (m + after);

  if (is_before) {
    step_length = -step_length;
    local_mean = 0.5 * (m + before);
  }

  var edge_uv: vec2<f32> = uv;

  if (is_horizontal) {
    edge_uv.y += step_length * 0.5;
  } else {
    edge_uv.x += step_length * 0.5;
  }

  // Walked along the edge each way until the luma leaves it.
  let along: vec2<f32> = select(vec2<f32>(0.0, texel.y), vec2<f32>(texel.x, 0.0), is_horizontal);
  var uv_back: vec2<f32> = edge_uv - along * STEPS[0];
  var uv_forward: vec2<f32> = edge_uv + along * STEPS[0];
  var end_back: f32 = luma_at(uv_back) - local_mean;
  var end_forward: f32 = luma_at(uv_forward) - local_mean;
  var is_done_back: bool = abs(end_back) >= gradient;
  var is_done_forward: bool = abs(end_forward) >= gradient;

  for (var index: u32 = 1u; index < SEARCH_STEPS; index++) {
    if (is_done_back && is_done_forward) {
      break;
    }

    if (!is_done_back) {
      uv_back -= along * STEPS[index];
      end_back = luma_at(uv_back) - local_mean;
      is_done_back = abs(end_back) >= gradient;
    }

    if (!is_done_forward) {
      uv_forward += along * STEPS[index];
      end_forward = luma_at(uv_forward) - local_mean;
      is_done_forward = abs(end_forward) >= gradient;
    }
  }

  let distance_back: f32 = select(uv.y - uv_back.y, uv.x - uv_back.x, is_horizontal);
  let distance_forward: f32 = select(uv_forward.y - uv.y, uv_forward.x - uv.x, is_horizontal);
  let is_back_nearer: bool = distance_back < distance_forward;
  let nearest: f32 = min(distance_back, distance_forward);
  let edge_length: f32 = distance_back + distance_forward;
  // The pixel is blended only where the edge's nearer end goes the other way from its centre.
  let is_centre_less: bool = m - local_mean < 0.0;
  let is_correct: bool = select(end_forward < 0.0, end_back < 0.0, is_back_nearer) != is_centre_less;
  let edge_offset: f32 = select(0.0, 0.5 - nearest / edge_length, is_correct);
  let offset: f32 = max(edge_offset, subpix);
  var final_uv: vec2<f32> = uv;

  if (is_horizontal) {
    final_uv.y += offset * step_length;
  } else {
    final_uv.x += offset * step_length;
  }

  return vec4<f32>(textureSampleLevel(frame, frame_sampler, final_uv, 0.0).rgb, center.a);
}
