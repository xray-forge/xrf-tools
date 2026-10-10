#import "common/noise"

// Overhead maps of the level around the camera, as `scene/level/overhead_map.rs` draws them straight down: for each
// column, how high the first thing over it stands. The rain's cover is one; the level's surface without its trees is
// another.

// Metres a point may stand under the first thing over its column and still take the rain: the depth's own error, never
// an overhang's height, as the engine's comparison has a few millimetres, so a pipe or a wall's top just under a
// ceiling stays dry. A point is read `COVER_OUT_TEXELS` out along its surface's lean, so a wall asks what stands over
// the air before it, a ceiling inside or the sky outside, and a slope reads down its fall; what that leaves of its own
// surface rising within the texel and a half its four texels reach is allowed as far as it rises, by its slope, at most
// `COVER_SLOPE_MOST`.
const COVER_BIAS: f32 = 0.03;
const COVER_SLOPE_TEXELS: f32 = 1.5;
const COVER_SLOPE_MOST: f32 = 2.0;
const COVER_OUT_TEXELS: f32 = 1.5;
// Metres under a point the rain may land and still creep to it, and how far the point's surface faces up for it to
// creep at all, none to all: water runs and splashes along the ground it lands on, never up a wall, which an overhead
// map cannot tell from a roof's edge, nor from a river or the ground outside a hull below it.
const COVER_CREEP_DROP: f32 = 0.5;
const COVER_CREEP_UP: vec2<f32> = vec2<f32>(0.5, 0.8);
// Rain's slant off the vertical at most, as its tangent (about 15 degrees), and the metres a cover must stand over a
// point before it lets slanted rain in (`slanted_reached`).
const COVER_SLANT: f32 = 0.27;
const COVER_SLANT_FROM: f32 = 6.0;
const SLANT_TAPS: u32 = 24u;
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

// How much reaches a point of what falls straight down, from none to one, by an overhead map and its pyramid
// (`scene/level/cover_pyramid.rs`), `shape` its texels across and the metres it reaches down, the point's surface
// facing `normal`. Where the point stands open, all of it, weighed over the four texels about it as `overhead_open`
// weighs them. Where something stands over a surface facing up, twice the share of the sky within `creep` metres of it
// open down to no more than `COVER_CREEP_DROP` under it, at most all: under a cover's edge as wet as the open ground beside it, dry
// `creep` under it, and wet throughout under a cover narrower than twice `creep`, as a long rain runs and splashes in
// under what is small. The pyramid tells first whether anything, or everything, stands over the point's surroundings,
// so only a point within `creep` of a cover's edge counts taps; `noise`, from nothing to one, turns them.
fn cover_reached(
  map: texture_depth_2d,
  pyramid: texture_2d<f32>,
  window: vec4<f32>,
  shape: vec2<f32>,
  surface: vec3<f32>,
  normal: vec3<f32>,
  creep: f32,
  noise: f32,
) -> f32 {
  let texel: f32 = window.z * 2.0 / shape.x;
  let across: f32 = length(normal.xz);
  let slope: f32 = select(COVER_SLOPE_MOST, min(across / max(normal.y, 1e-3), COVER_SLOPE_MOST), normal.y > 0.0);
  let point: vec3<f32> = surface + vec3<f32>(normal.x, 0.0, normal.z) * (texel * COVER_OUT_TEXELS);
  let bias: f32 = COVER_BIAS + texel * slope * max(COVER_SLOPE_TEXELS - COVER_OUT_TEXELS * across, 0.0);
  let own: f32 = overhead_open(map, window, shape.x, shape.y, point, bias);
  let spread: f32 = creep * smoothstep(COVER_CREEP_UP.x, COVER_CREEP_UP.y, normal.y);

  let slanted: f32 = select(slanted_reached(map, window, shape, point), 0.0, own >= 1.0);

  if (own >= 1.0 || spread <= texel) {
    return max(own, slanted);
  }

  let limit: f32 = point.y + bias;
  let deepest: f32 = point.y - COVER_CREEP_DROP;
  // The level whose texels span the creep's diameter, so two by two of them hold every column within it.
  let levels: i32 = i32(textureNumLevels(pyramid));
  let level: i32 = clamp(i32(ceil(log2(2.0 * spread / texel))) - 1, 0, levels - 1);
  let span: f32 = texel * exp2(f32(level + 1));
  let first: vec2<i32> = vec2<i32>(floor((point.xz - window.xy + window.z - spread) / span));
  let last: vec2<i32> = vec2<i32>(textureDimensions(pyramid, level)) - 1;
  var lowest: f32 = 1.0;
  var highest: f32 = 0.0;

  for (var corner: u32 = 0u; corner < 4u; corner++) {
    let at: vec2<i32> = clamp(first + vec2<i32>(i32(corner & 1u), i32(corner >> 1u)), vec2<i32>(0), last);
    let range: vec2<f32> = textureLoad(pyramid, at, level).xy;

    lowest = min(lowest, range.x);
    highest = max(highest, range.y);
  }

  // Reversed: the highest thing stands at the largest depth. Open everywhere about it within the drop, or covered
  // everywhere, or open only far under it.
  let top: f32 = window.w - (1.0 - highest) * shape.y;
  let bottom: f32 = window.w - (1.0 - lowest) * shape.y;

  if (top <= limit && bottom >= deepest) {
    return 1.0;
  }

  if (bottom > limit || top < deepest) {
    return own;
  }

  var open: f32 = 0.0;

  for (var tap: u32 = 0u; tap < COVER_TAPS; tap++) {
    let along: f32 = sqrt((f32(tap) + 0.5) / f32(COVER_TAPS)) * spread;
    let turn: f32 = f32(tap) * COVER_GOLDEN_ANGLE + noise * 6.2831853;
    let column: vec3<f32> = point + vec3<f32>(cos(turn) * along, 0.0, sin(turn) * along);

    let landing: f32 = overhead_height(map, window, shape.x, shape.y, column);

    open += select(0.0, 1.0, landing <= limit && landing >= deepest);
  }

  return max(max(own, slanted), saturate(2.0 * open / f32(COVER_TAPS)));
}

// How much slanted rain reaches a covered point under a high cover, from none to one: rain falls up to `COVER_SLANT`
// off the vertical, so a cover more than `COVER_SLANT_FROM` metres over the point lets it in from the open sky within
// as far again, a share of its height past that. Twice the share of points within that reach with nothing standing
// over them higher than `COVER_SLANT_FROM` over the point, at most all, over `SLANT_TAPS` points laid alike for every pixel, so
// the share steps finely rather than grains. Only past a height no room's ceiling reaches, as the map sees no walls:
// the ground under a bridge's deck took the deck's outline sharp and dry, however high it stood.
fn slanted_reached(map: texture_depth_2d, window: vec4<f32>, shape: vec2<f32>, point: vec3<f32>) -> f32 {
  let cover: f32 = overhead_height(map, window, shape.x, shape.y, point);
  let reach: f32 = (cover - point.y - COVER_SLANT_FROM) * COVER_SLANT;

  if (reach <= 0.0) {
    return 0.0;
  }

  var open: f32 = 0.0;

  for (var tap: u32 = 0u; tap < SLANT_TAPS; tap++) {
    let along: f32 = sqrt((f32(tap) + 0.5) / f32(SLANT_TAPS)) * reach;
    let turn: f32 = f32(tap) * COVER_GOLDEN_ANGLE;
    let column: vec3<f32> = point + vec3<f32>(cos(turn) * along, 0.0, sin(turn) * along);

    open += select(0.0, 1.0, overhead_height(map, window, shape.x, shape.y, column) < point.y + COVER_SLANT_FROM);
  }

  return saturate(2.0 * open / f32(SLANT_TAPS));
}
