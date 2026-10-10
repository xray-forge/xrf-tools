// Overhead maps of the level around the camera, as `scene/level/overhead_map.rs` draws them straight down: for each
// column, how high the first thing over it stands. The rain's cover is one; the level's surface without its trees is
// another.

// Metres a point may stand under the first thing over its column and still take the rain, at least; and as far again
// as a slope of one rises over a texel and a half, the farthest the texels a point is weighed between stand from it.
const COVER_BIAS: f32 = 0.2;
const COVER_SLOPE_TEXELS: f32 = 1.5;
// Taps the open sky about a covered point is counted by, laid out as a golden-angle spiral over the creep's disc.
const COVER_TAPS: u32 = 12u;
const COVER_GOLDEN_ANGLE: f32 = 2.3999632;

// The height of the first thing over a column in an overhead map, in renderer space: read from its depth, reversed,
// straight down from the height it is seen from; nothing at all outside it. `window` is its centre in `x` and `z`, its
// half width, and the height it is seen from; `resolution` its texels across and `reach` the metres it reaches down.
fn overhead_height(map: texture_depth_2d, window: vec4<f32>, resolution: f32, reach: f32, column: vec3<f32>) -> f32 {
  let uv: vec2<f32> = vec2<f32>(column.x - window.x, column.z - window.y) / (window.z * 2.0) + 0.5;

  if (any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
    return -1e9;
  }

  let texel: vec2<i32> = vec2<i32>(clamp(uv * resolution, vec2<f32>(0.0), vec2<f32>(resolution - 1.0)));

  return window.w - (1.0 - textureLoad(map, texel, 0)) * reach;
}

// How open a point is to what falls straight down, by an overhead map laid out as `overhead_height` reads one: each of
// the four texels about it open where the point stands no more than `bias` metres under the first thing over that
// column, weighed by how near the point is to each, so a cover's edge softens over a texel and never steps; open where
// the map does not reach.
fn overhead_open(
  map: texture_depth_2d,
  window: vec4<f32>,
  resolution: f32,
  reach: f32,
  point: vec3<f32>,
  bias: f32,
) -> f32 {
  let uv: vec2<f32> = vec2<f32>(point.x - window.x, point.z - window.y) / (window.z * 2.0) + 0.5;

  if (any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0))) {
    return 1.0;
  }

  let at: vec2<f32> = uv * resolution - 0.5;
  let first: vec2<f32> = floor(at);
  let weight: vec2<f32> = at - first;
  let last: vec2<i32> = vec2<i32>(i32(resolution) - 1);
  var open: vec4<f32>;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let texel: vec2<i32> = clamp(vec2<i32>(first) + vec2<i32>(i32(corner & 1u), i32(corner >> 1u)), vec2<i32>(0), last);
    let height: f32 = window.w - (1.0 - textureLoad(map, texel, 0)) * reach;

    open[corner] = select(0.0, 1.0, point.y + bias >= height);
  }

  return mix(mix(open.x, open.y, weight.x), mix(open.z, open.w, weight.x), weight.y);
}

// Interleaved gradient noise: one value a pixel, from zero to one, which turns the creep's taps.
fn cover_noise(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

// How much reaches a point of what falls straight down, from none to one, by an overhead map and its pyramid
// (`scene/level/cover_pyramid.rs`), `shape` its texels across and the metres it reaches down. Where the point stands
// open, all of it, weighed over the four texels about it as `overhead_open` weighs them. Where something stands over
// it, twice the share of the open sky within `creep` metres of it, at most all: under a cover's edge as wet as the open
// ground beside it, dry `creep` under it, and wet throughout under a cover narrower than twice `creep`, as a long rain
// runs and splashes in under what is small. The pyramid tells first whether anything, or everything, stands over the
// point's surroundings, so only a point within `creep` of a cover's edge counts taps; `noise`, from nothing to one,
// turns them.
fn cover_reached(
  map: texture_depth_2d,
  pyramid: texture_2d<f32>,
  window: vec4<f32>,
  shape: vec2<f32>,
  point: vec3<f32>,
  creep: f32,
  noise: f32,
) -> f32 {
  let texel: f32 = window.z * 2.0 / shape.x;
  let bias: f32 = COVER_BIAS + texel * COVER_SLOPE_TEXELS;
  let own: f32 = overhead_open(map, window, shape.x, shape.y, point, bias);

  if (own >= 1.0 || creep <= texel) {
    return own;
  }

  let limit: f32 = point.y + bias;
  // The level whose texels span the creep's diameter, so two by two of them hold every column within it.
  let levels: i32 = i32(textureNumLevels(pyramid));
  let level: i32 = clamp(i32(ceil(log2(2.0 * creep / texel))) - 1, 0, levels - 1);
  let span: f32 = texel * exp2(f32(level + 1));
  let first: vec2<i32> = vec2<i32>(floor((point.xz - window.xy + window.z - creep) / span));
  let last: vec2<i32> = vec2<i32>(textureDimensions(pyramid, level)) - 1;
  var lowest: f32 = 1.0;
  var highest: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let at: vec2<i32> = clamp(first + vec2<i32>(i32(corner & 1u), i32(corner >> 1u)), vec2<i32>(0), last);
    let range: vec2<f32> = textureLoad(pyramid, at, level).xy;

    lowest = min(lowest, range.x);
    highest = max(highest, range.y);
  }

  // Reversed: the highest thing stands at the largest depth.
  if (window.w - (1.0 - highest) * shape.y <= limit) {
    return 1.0;
  }

  if (window.w - (1.0 - lowest) * shape.y > limit) {
    return own;
  }

  var open: f32 = 0.0;

  for (var tap: u32 = 0u; tap < COVER_TAPS; tap++) {
    let along: f32 = sqrt((f32(tap) + 0.5) / f32(COVER_TAPS)) * creep;
    let turn: f32 = f32(tap) * COVER_GOLDEN_ANGLE + noise * 6.2831853;
    let column: vec3<f32> = point + vec3<f32>(cos(turn) * along, 0.0, sin(turn) * along);

    open += select(0.0, 1.0, limit >= overhead_height(map, window, shape.x, shape.y, column));
  }

  return max(own, saturate(2.0 * open / f32(COVER_TAPS)));
}
