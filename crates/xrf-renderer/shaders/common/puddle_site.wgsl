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

// How far out a point with no puddle about it stands: past any radius.
const SITE_NONE: f32 = 1e9;

// How far out from a puddle's site a point `world` stands, in renderer space as the sites are, a share of that
// puddle's radius: the least over the puddles of its cell and those about it, each the stretched, turned blob the
// wetting draws about its site (`site_puddle` in `frame/wet_patch.wgsl`, without the noise that moves its border),
// among those the point stands level with, within the band a site holds water over and `slack` metres more;
// `SITE_NONE` where none does. `sites` is the frame's puddles, a site a cell `cell` metres across, over the level
// surface's window `surface`.
fn site_reach(sites: texture_2d<f32>, surface: vec4<f32>, cell: f32, world: vec3<f32>, slack: f32) -> f32 {
  let cells: f32 = floor(surface.z * 2.0 / cell);
  let origin: vec2<f32> = site_origin(surface.xy - surface.z, cell);
  let at: vec2<f32> = floor(world.xz / cell) - origin;
  var reach: f32 = SITE_NONE;

  for (var y: i32 = -1; y <= 1; y++) {
    for (var x: i32 = -1; x <= 1; x++) {
      let near: vec2<f32> = at + vec2<f32>(f32(x), f32(y));

      if (any(near < vec2<f32>(0.0)) || any(near >= vec2<f32>(cells))) {
        continue;
      }

      let site: vec4<f32> = textureLoad(sites, vec2<i32>(near), 0);

      if (site.w <= 0.0 || length(world.xz - site.xy) > site.w * 2.0) {
        continue;
      }

      let hash: vec3<f32> = site_hash(origin + near);
      let turn: f32 = hash.y * 6.2831853;
      let across: vec2<f32> = vec2<f32>(cos(turn), sin(turn));
      let offset: vec2<f32> = world.xz - site.xy;
      let stretched: vec2<f32> = vec2<f32>(dot(offset, across), dot(offset, vec2<f32>(-across.y, across.x))) /
        vec2<f32>(1.0, mix(1.0, 1.0 / PUDDLE_STRETCH, hash.z));

      if (abs(world.y - site.z) < SITE_LEVEL.x + SITE_LEVEL.y * site.w + slack) {
        reach = min(reach, length(stretched) / site.w);
      }
    }
  }

  return reach;
}
