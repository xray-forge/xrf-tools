#import "common/fullscreen"

// SMAA 1x, medium preset with colour edge detection, v2.8 (MIT, Jorge Jimenez, Jose I. Echevarria, Belen Masia,
// Fernando Navarro, Diego Gutierrez), as three.js's `SMAANode` stages it: the frame's edges found, the area each edge
// pattern covers looked up for every edge pixel, and each pixel blended with its neighbour across the strongest.

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var edges_texture: texture_2d<f32>;
@group(0) @binding(2) var weights_texture: texture_2d<f32>;
@group(0) @binding(3) var area_texture: texture_2d<f32>;
@group(0) @binding(4) var search_texture: texture_2d<f32>;
@group(0) @binding(5) var linear_sampler: sampler;
@group(0) @binding(6) var point_sampler: sampler;

const THRESHOLD: f32 = 0.1;
const MAX_SEARCH_STEPS: i32 = 8;
const AREATEX_MAX_DISTANCE: f32 = 16.0;
const AREATEX_PIXEL_SIZE: vec2<f32> = vec2<f32>(1.0 / 160.0, 1.0 / 560.0);
const AREATEX_SUBTEX_SIZE: f32 = 1.0 / 7.0;

fn inv_size() -> vec2<f32> {
  return 1.0 / vec2<f32>(textureDimensions(source));
}

fn sample_source(at: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(source, linear_sampler, at, 0.0);
}

fn sample_edges(at: vec2<f32>) -> vec2<f32> {
  return textureSampleLevel(edges_texture, linear_sampler, at, 0.0).rg;
}

fn max3(value: vec3<f32>) -> f32 {
  return max(value.r, max(value.g, value.b));
}

@fragment
fn fs_smaa_edges(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = inv_size();
  let uv: vec2<f32> = in.clip.xy * size;
  let offset0: vec4<f32> = uv.xyxy + size.xyxy * vec4<f32>(-1.0, 0.0, 0.0, -1.0);
  let offset1: vec4<f32> = uv.xyxy + size.xyxy * vec4<f32>(1.0, 0.0, 0.0, 1.0);
  let offset2: vec4<f32> = uv.xyxy + size.xyxy * vec4<f32>(-2.0, 0.0, 0.0, -2.0);
  let c: vec3<f32> = sample_source(uv).rgb;
  var delta: vec4<f32> = vec4<f32>(0.0);

  // The left and top deltas, against the threshold; nothing more where there is no edge.
  delta.x = max3(abs(c - sample_source(offset0.xy).rgb));
  delta.y = max3(abs(c - sample_source(offset0.zw).rgb));

  var edges: vec2<f32> = step(vec2<f32>(THRESHOLD), delta.xy);

  if (dot(edges, vec2<f32>(1.0)) == 0.0) {
    discard;
  }

  // The right and bottom, then the left-left and top-top deltas, for the local contrast adaptation.
  delta.z = max3(abs(c - sample_source(offset1.xy).rgb));
  delta.w = max3(abs(c - sample_source(offset1.zw).rgb));

  var max_delta: f32 = max(max(delta.x, delta.y), max(delta.z, delta.w));

  delta.z = max3(abs(c - sample_source(offset2.xy).rgb));
  delta.w = max3(abs(c - sample_source(offset2.zw).rgb));
  max_delta = max(max_delta, max(delta.z, delta.w));
  edges *= step(vec2<f32>(0.5 * max_delta), delta.xy);

  return vec4<f32>(edges, 0.0, 0.0);
}

fn search_length(e: vec2<f32>, bias: f32, scale: f32) -> f32 {
  var coord: vec2<f32> = e;

  coord.x = bias + coord.x * scale;

  return 255.0 * textureSampleLevel(search_texture, point_sampler, coord, 0.0).r;
}

fn area(distance: vec2<f32>, e1: f32, e2: f32, offset: f32) -> vec2<f32> {
  // Rounding prevents precision errors of bilinear filtering; then a scale and bias into texel space, moved to the
  // subpixel offset's place.
  var texcoord: vec2<f32> = AREATEX_MAX_DISTANCE * round(4.0 * vec2<f32>(e1, e2)) + distance;

  texcoord = AREATEX_PIXEL_SIZE * texcoord + 0.5 * AREATEX_PIXEL_SIZE;
  texcoord.y += AREATEX_SUBTEX_SIZE * offset;

  return textureSampleLevel(area_texture, linear_sampler, texcoord, 0.0).rg;
}

// The four searches along an edge, each sampling between edges to fetch four at once (`@PSEUDO_GATHER4`).
fn search_x_left(start: vec2<f32>, end: f32, size: vec2<f32>) -> f32 {
  var e: vec2<f32> = vec2<f32>(0.0, 1.0);
  var coord: vec2<f32> = start;

  for (var index: i32 = 0; index < MAX_SEARCH_STEPS; index++) {
    e = sample_edges(coord);
    coord -= vec2<f32>(2.0, 0.0) * size;

    if (coord.x <= end || e.g <= 0.8281 || e.r != 0.0) {
      break;
    }
  }

  coord.x += 0.25 * size.x + size.x + 2.0 * size.x;
  coord.x -= size.x * search_length(e, 0.0, 0.5);

  return coord.x;
}

fn search_x_right(start: vec2<f32>, end: f32, size: vec2<f32>) -> f32 {
  var e: vec2<f32> = vec2<f32>(0.0, 1.0);
  var coord: vec2<f32> = start;

  for (var index: i32 = 0; index < MAX_SEARCH_STEPS; index++) {
    e = sample_edges(coord);
    coord += vec2<f32>(2.0, 0.0) * size;

    if (coord.x >= end || e.g <= 0.8281 || e.r != 0.0) {
      break;
    }
  }

  coord.x -= 0.25 * size.x + size.x + 2.0 * size.x;
  coord.x += size.x * search_length(e, 0.5, 0.5);

  return coord.x;
}

fn search_y_up(start: vec2<f32>, end: f32, size: vec2<f32>) -> f32 {
  var e: vec2<f32> = vec2<f32>(1.0, 0.0);
  var coord: vec2<f32> = start;

  for (var index: i32 = 0; index < MAX_SEARCH_STEPS; index++) {
    e = sample_edges(coord);
    coord += vec2<f32>(0.0, -2.0) * size;

    if (coord.y <= end || e.r <= 0.8281 || e.g != 0.0) {
      break;
    }
  }

  coord.y += 0.25 * size.y + size.y + 2.0 * size.y;
  coord.y -= size.y * search_length(e.gr, 0.0, 0.5);

  return coord.y;
}

fn search_y_down(start: vec2<f32>, end: f32, size: vec2<f32>) -> f32 {
  var e: vec2<f32> = vec2<f32>(1.0, 0.0);
  var coord: vec2<f32> = start;

  for (var index: i32 = 0; index < MAX_SEARCH_STEPS; index++) {
    e = sample_edges(coord);
    coord -= vec2<f32>(0.0, -2.0) * size;

    if (coord.y >= end || e.r <= 0.8281 || e.g != 0.0) {
      break;
    }
  }

  coord.y -= 0.25 * size.y + size.y + 2.0 * size.y;
  coord.y += size.y * search_length(e.gr, 0.5, 0.5);

  return coord.y;
}

@fragment
fn fs_smaa_weights(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = inv_size();
  let uv: vec2<f32> = in.clip.xy * size;
  let pixel: vec2<f32> = uv / size;
  let offset0: vec4<f32> = uv.xyxy + size.xyxy * vec4<f32>(-0.25, -0.125, 1.25, -0.125);
  let offset1: vec4<f32> = uv.xyxy + size.xyxy * vec4<f32>(-0.125, -0.25, -0.125, 1.25);
  // Where the searches end.
  let offset2: vec4<f32> = vec4<f32>(offset0.xz, offset1.yw)
    + vec4<f32>(-2.0, 2.0, -2.0, 2.0) * vec4<f32>(size.xx, size.yy) * f32(MAX_SEARCH_STEPS);
  var weights: vec4<f32> = vec4<f32>(0.0);
  let e: vec2<f32> = sample_edges(uv);

  // An edge at the north: the distances to its left and right ends, and the crossing edges there.
  if (e.g > 0.0) {
    let left: vec2<f32> = vec2<f32>(search_x_left(offset0.xy, offset2.x, size), offset1.y);
    let right: vec2<f32> = vec2<f32>(search_x_right(offset0.zw, offset2.y, size), offset1.y);
    let distance: vec2<f32> = vec2<f32>(left.x, right.x) / size.x - pixel.x;
    let e1: f32 = sample_edges(left).r;
    let e2: f32 = sample_edges(right + vec2<f32>(1.0, 0.0) * size).r;

    weights = vec4<f32>(area(sqrt(abs(distance)), e1, e2, 0.0), weights.ba);
  }

  // An edge at the west: the distances to its top and bottom ends.
  if (e.r > 0.0) {
    let up: vec2<f32> = vec2<f32>(offset0.x, search_y_up(offset1.xy, offset2.z, size));
    let down: vec2<f32> = vec2<f32>(offset0.x, search_y_down(offset1.zw, offset2.w, size));
    let distance: vec2<f32> = vec2<f32>(up.y, down.y) / size.y - pixel.y;
    let e1: f32 = sample_edges(up).g;
    let e2: f32 = sample_edges(down + vec2<f32>(0.0, 1.0) * size).g;

    weights = vec4<f32>(weights.rg, area(sqrt(abs(distance)), e1, e2, 0.0));
  }

  return weights;
}

@fragment
fn fs_smaa_blend(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let size: vec2<f32> = inv_size();
  let uv: vec2<f32> = in.clip.xy * size;
  let offset1: vec4<f32> = uv.xyxy + size.xyxy * vec4<f32>(1.0, 0.0, 0.0, 1.0);
  let here: vec4<f32> = textureSampleLevel(weights_texture, linear_sampler, uv, 0.0);
  let a: vec4<f32> = vec4<f32>(
    here.x,
    textureSampleLevel(weights_texture, linear_sampler, offset1.zw, 0.0).g,
    here.z,
    textureSampleLevel(weights_texture, linear_sampler, offset1.xy, 0.0).a,
  );
  let c: vec4<f32> = sample_source(uv);

  if (dot(a, vec4<f32>(1.0)) < 1e-5) {
    return c;
  }

  // Up to four lines cross a pixel, one through each edge: the strongest each way, then the stronger direction.
  var offset: vec2<f32> = vec2<f32>(select(-a.b, a.a, a.a > a.b), select(-a.r, a.g, a.g > a.r));

  if (abs(offset.x) > abs(offset.y)) {
    offset.y = 0.0;
  } else {
    offset.x = 0.0;
  }

  let opposite: vec4<f32> = sample_source(uv + sign(offset) * size);
  let share: f32 = select(abs(offset.y), abs(offset.x), abs(offset.x) > abs(offset.y));

  return mix(c, opposite, share);
}
