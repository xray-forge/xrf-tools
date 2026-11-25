#import "common/math"

// The enhanced foliage motion: a tree's trunk swinging downwind by its height squared, its branches and every waving
// tuft carried by a flow field drifting with the wind, all of it in a wind that gusts over the level. Everything works
// in the engine's space, where the renderer's `z` is negated; the setup is passed in, so the trees' and the grass's
// programs read it from their own uniforms.

// The flow field's cells a unit of its coordinates, and its gust layer's, stretched along the wind's `x`.
const FLOW_CELLS: f32 = 12.0;
const GUST_CELLS: vec2<f32> = vec2<f32>(4.0, 10.0);

// The gusts' two layers: the share of their spacing a cell of each stands along the wind, which
// `FoliageWind::GUST_PACE` moves it by, and across it.
const GUST_BROAD: vec2<f32> = vec2<f32>(1.0, 2.5);
const GUST_MIDDLE: vec2<f32> = vec2<f32>(0.45, 0.9);
// The cells the gusts' noise repeats over along the wind, a power of two, as a mask (`FoliageWind::GUST_REPEAT`), so
// their move downwind is taken within them; across the wind it never repeats.
const GUST_MASK: vec2<u32> = vec2<u32>(1023u, 0xffffffffu);
// How much of the wind is left between two gusts, and how far past it one blows.
const GUST_CALM: f32 = 0.3;
const GUST_DRIVEN: f32 = 1.5;
// The furthest a gust tilts a tuft over, radians.
const GUST_TILT: f32 = 1.1;

// What a tree's or a tuft's motion reads: the wind's way across the ground in `xy`, its speed in `z` and that speed's
// root in `w`; the grass's drift speed, turbulence, push and wave; the branches' drift speed, the trunks' swing speed
// and bend; how far the fields have drifted, `xy` across the ground and `z` the trunks' clock; and the gusts' strength,
// the metres apart they stand, and the cells their broad and their smaller layer have moved downwind.
struct FoliageSetup {
  wind: vec4<f32>,
  grass: vec4<f32>,
  trees: vec4<f32>,
  anim: vec4<f32>,
  gusts: vec4<f32>,
};

// A unit gradient for a cell, its angle from an integer hash of the cell (xxHash32's avalanche). A vertex's place is
// worked out twice a frame, where it stands and where it stood, and the two must agree to the bit wherever the fields
// have not moved: the usual `fract(sin(x) × 43758.5)` hash does not, since a last-place difference between the two
// compiled reads of a large `sin` turns a cell's gradient anywhere, which wrote steady false motion over whole bushes
// and grained the plants under every temporal resolve. `cell_mask` holds the cells along each axis the noise repeats
// over.
fn foliage_gradient(cell: vec2<f32>, cell_mask: vec2<u32>) -> vec2<f32> {
  let key: vec2<u32> = bitcast<vec2<u32>>(vec2<i32>(cell)) & cell_mask;
  var hash: u32 = key.x * 1664525u + key.y * 1013904223u + 374761393u;

  hash = (hash ^ (hash >> 16u)) * 2246822519u;
  hash = (hash ^ (hash >> 13u)) * 3266489917u;
  hash = hash ^ (hash >> 16u);

  let angle: f32 = f32(hash >> 8u) / 16777216.0 * TAU;

  return vec2<f32>(cos(angle), sin(angle));
}

// Gradient noise at a point and its slope there, which never repeats.
fn foliage_noise(at: vec2<f32>) -> vec3<f32> {
  return foliage_noise_within(at, vec2<u32>(0xffffffffu));
}

// Gradient noise at a point and its slope there, repeating over the cells `cell_mask` holds: quintic, so the slope
// is smooth across cells.
fn foliage_noise_within(at: vec2<f32>, cell_mask: vec2<u32>) -> vec3<f32> {
  let cell: vec2<f32> = floor(at);
  let f: vec2<f32> = at - cell;
  let u: vec2<f32> = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  let du: vec2<f32> = 30.0 * f * f * (f * (f - 2.0) + 1.0);
  let ga: vec2<f32> = foliage_gradient(cell, cell_mask);
  let gb: vec2<f32> = foliage_gradient(cell + vec2<f32>(1.0, 0.0), cell_mask);
  let gc: vec2<f32> = foliage_gradient(cell + vec2<f32>(0.0, 1.0), cell_mask);
  let gd: vec2<f32> = foliage_gradient(cell + vec2<f32>(1.0, 1.0), cell_mask);
  let va: f32 = dot(ga, f);
  let vb: f32 = dot(gb, f - vec2<f32>(1.0, 0.0));
  let vc: f32 = dot(gc, f - vec2<f32>(0.0, 1.0));
  let vd: f32 = dot(gd, f - vec2<f32>(1.0, 1.0));
  let k: f32 = va - vb - vc + vd;
  let value: f32 = va + u.x * (vb - va) + u.y * (vc - va) + u.x * u.y * k;
  let slope: vec2<f32> = ga + u.x * (gb - ga) + u.y * (gc - ga) + u.x * u.y * (ga - gb - gc + gd) +
    du * (u.yx * k + vec2<f32>(vb, vc) - va);

  return vec3<f32>(value, slope);
}

// The flow field at a point, each channel from nothing to one around a half: `xy` how far it tosses across the
// ground, the slopes of rolling bumps; `z` its gusts, a broad noise stretched along `x`.
fn foliage_flow(at: vec2<f32>) -> vec3<f32> {
  let bumps: vec3<f32> = foliage_noise(at * FLOW_CELLS);
  let gust: f32 = foliage_noise(at * GUST_CELLS).x + 0.5 * foliage_noise(at * GUST_CELLS * 2.03 + 17.0).x;

  return vec3<f32>(saturate(bumps.yz * 0.25 + 0.5), saturate(gust * 0.7 + 0.5));
}

// A trunk's swing at a height over its foot: across the ground in `xy`, and its gust, from nothing to one, in `z`.
// The swing's phase starts at the foot's height, so neighbours swing apart.
fn foliage_trunk(foot: f32, height: f32, setup: FoliageSetup) -> vec3<f32> {
  let phase: f32 = foot + setup.anim.z * setup.trees.y;
  let swing: f32 = (cos(phase) * sin(phase * 5.0) + 0.5) * setup.trees.z;
  let speed: f32 = saturate(setup.wind.w * 1.5);
  let bend: f32 = speed * 0.006 * saturate(1.0 - height * 0.005) * height * height * swing * speed;

  return vec3<f32>(bend * setup.wind.xy, saturate((swing + 1.0) * 0.5));
}

// A branch's or a leaf's move at a place on the ground, a height over its tree's foot and its texture's `v`: tossed by
// the flow field, pushed downwind by its gusts, as much as its height down the texture lets it, over its trunk's swing.
fn foliage_branches(at: vec2<f32>, foot: f32, height: f32, v: f32, setup: FoliageSetup) -> vec3<f32> {
  let offset: vec2<f32> = -setup.anim.xy * setup.trees.x;
  let flow: vec3<f32> = foliage_flow((at + offset) * 0.02);
  let detail: vec3<f32> = foliage_flow((at + offset * 0.2) * 0.1);
  var motion: vec3<f32> = vec3<f32>(flow.x, detail.y, flow.y) * 2.0 - 1.0;
  let trunk: vec3<f32> = foliage_trunk(foot, height, setup);

  motion = vec3<f32>(motion.xz * trunk.z * clamp(height * 0.1, 1.0, 2.5) + detail.z * setup.wind.xy, motion.y).xzy;
  motion.y *= saturate(height * 0.1);
  motion *= (1.0 - v) * setup.wind.z;

  return motion + vec3<f32>(trunk.x, 0.0, trunk.y);
}

// How far a gust has risen at a place, from nothing between two to one in it: noise stretched across the wind, so its
// crests stand across it, travelling downwind, the smaller layer riding the larger.
fn foliage_gust(at: vec2<f32>, setup: FoliageSetup) -> f32 {
  let wind: vec2<f32> = setup.wind.xy;
  let place: vec2<f32> = vec2<f32>(dot(at, wind), dot(at, vec2<f32>(-wind.y, wind.x)));
  let broad_at: vec2<f32> = gust_coordinate(place, setup.gusts.y * GUST_BROAD, setup.gusts.z);
  let broad: f32 = foliage_noise_within(broad_at, GUST_MASK).x;
  let middle_at: vec2<f32> = gust_coordinate(place, setup.gusts.y * GUST_MIDDLE, setup.gusts.w);
  let middle: f32 = foliage_noise_within(middle_at + vec2<f32>(17.0, broad), GUST_MASK).x;

  return smoothstep(-0.05, 0.4, broad * 0.9 + middle * 0.45);
}

// Where a layer of the gusts reads its noise at a place along and across the wind, its cell so many metres each way,
// `moved` so many cells downwind.
fn gust_coordinate(place: vec2<f32>, cell: vec2<f32>, moved: f32) -> vec2<f32> {
  return place / cell - vec2<f32>(moved, 0.0);
}

// The setup at a place, its wind calmed or driven by the gust `risen` there, as far as the gusts are strong.
fn foliage_gusted(setup: FoliageSetup, risen: f32) -> FoliageSetup {
  let speed: f32 = setup.wind.z * mix(1.0, mix(GUST_CALM, GUST_DRIVEN, risen), saturate(setup.gusts.x));
  var gusted: FoliageSetup = setup;

  gusted.wind = vec4<f32>(setup.wind.xy, speed, saturate(sqrt(speed * 1.66)));

  return gusted;
}

// A waving tuft's vertex move at a place on the ground and a height over the tuft's foot: tossed by the flow field,
// pushed downwind and lifted by the field's gusts, the shortest and the tallest held stiffer, then tilted downwind by a
// wind's gust crossing it.
fn foliage_grass(at: vec2<f32>, height: f32, setup: FoliageSetup) -> vec3<f32> {
  if (setup.gusts.x <= 0.0) {
    return foliage_tossed(at, height, setup);
  }

  let risen: f32 = foliage_gust(at, setup);
  let gusted: FoliageSetup = foliage_gusted(setup, risen);
  // Squared, so a gust stands out from the calm between two.
  let tilt: f32 = min(risen * risen * gusted.wind.z * setup.gusts.x, GUST_TILT);
  let lean: vec2<f32> = setup.wind.xy * sin(tilt) * height;

  return foliage_tossed(at, height, gusted) + vec3<f32>(lean.x, (cos(tilt) - 1.0) * height, lean.y);
}

// A waving tuft's vertex tossed by the flow field, pushed downwind and lifted by its gusts.
fn foliage_tossed(at: vec2<f32>, height: f32, setup: FoliageSetup) -> vec3<f32> {
  let limit: f32 = saturate(height * height - 0.01) * saturate(1.0 - height * 0.1);
  let flow: vec3<f32> = foliage_flow((at - setup.anim.xy * setup.grass.x) * 0.018);
  let toss: vec2<f32> = (flow.xy * 2.0 - 1.0) * setup.grass.y;
  let push: vec2<f32> = flow.z * setup.wind.xy * setup.grass.z;

  return vec3<f32>(toss.x + push.x, flow.z * setup.grass.w, toss.y + push.y) * setup.wind.z * limit;
}
