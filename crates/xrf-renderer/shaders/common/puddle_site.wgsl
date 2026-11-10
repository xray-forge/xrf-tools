#import "common/noise"

// What placing the puddles and drawing them share about a site: each world cell of the level surface holds at most
// one, its keep, shape and jitter drawn from one hash of the cell.

// How far the surface may rise and fall across a site and still hold water: a little, and a share of its radius, a
// gentle slope of some four degrees, which a puddle's water still lies on.
const SITE_LEVEL: vec2<f32> = vec2<f32>(0.15, 0.07);
// A puddle's shape about its site: as long as wide at most by this, turned and stretched by the site's hash.
const PUDDLE_STRETCH: f32 = 1.6;
// The share of a puddle's radius within which no tuft stands whole, short of its soft rim where the ground is only
// damp, and the share in from there over which a tuft shrinks to nothing, so a growing puddle takes it smoothly.
const PUDDLE_BARE: f32 = 0.85;
const PUDDLE_SHORE: f32 = 0.15;
// The share of the puddles' distance over which they fade out.
const PUDDLE_FADE: f32 = 0.15;

// How much of a puddle is drawn `from_eye` metres from the eye, the puddles drawn to `distance`: all short of their
// fade, none past it.
fn puddle_drawn(from_eye: f32, distance: f32) -> f32 {
  return smoothstep(distance, distance * (1.0 - PUDDLE_FADE), from_eye);
}

// The share of its size a tuft stands at by one puddle: `across` how far out from the site its foot stands, a share of
// the puddle's radius, and `drawn` how much of the puddle is drawn there (`puddle_drawn`), so no tuft is missing where
// no water shows.
fn puddle_tuft(across: f32, drawn: f32) -> f32 {
  return 1.0 - drawn * smoothstep(PUDDLE_BARE, PUDDLE_BARE - PUDDLE_SHORE, across);
}

// A hash of a world cell, from nothing to one each way, three ways: `x` ranks the site's keep, `y` and `z` turn and
// stretch its puddle.
fn site_hash(cell: vec2<f32>) -> vec3<f32> {
  var mixed: vec3<f32> = fract(vec3<f32>(cell.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));

  mixed += dot(mixed, mixed.yxz + 33.33);

  return fract((mixed.xxy + mixed.yzz) * mixed.zyx);
}

// Where a site stands in its world cell, from nothing to one each way: a hash of its own, so its spot does not follow
// its keep or its shape.
fn site_jitter(cell: vec2<f32>) -> vec2<f32> {
  return hash22(cell);
}

// The world cell, whole metres over the cell size, of the first cell of a map whose corner is `corner`: the hashes,
// the spots and the lookups are all of world cells, so a site keeps its place and its puddle as the map moves, by a
// step that need be no whole number of cells.
fn site_origin(corner: vec2<f32>, cell: f32) -> vec2<f32> {
  return floor(corner / cell + 0.5);
}
