// The enhanced foliage motion: a tree's trunk swinging downwind by its height squared, its branches and every waving
// tuft carried by a flow field drifting with the wind. Everything works in the engine's space, where the renderer's
// `z` is negated; the setup is passed in, so the trees' and the grass's programs read it from their own uniforms.

const FOLIAGE_TAU: f32 = 6.2831853;

// The flow field's cells a unit of its coordinates, and its gust layer's, stretched along the wind's `x`.
const FLOW_CELLS: f32 = 12.0;
const GUST_CELLS: vec2<f32> = vec2<f32>(4.0, 10.0);

// What a tree's or a tuft's motion reads: the wind's way across the ground in `xy`, its speed in `z` and that speed's
// root in `w`; the grass's drift speed, turbulence, push and wave; the branches' drift speed, the trunks' swing speed
// and bend; and how far the fields have drifted, `xy` across the ground and `z` the trunks' clock.
struct FoliageSetup {
  wind: vec4<f32>,
  grass: vec4<f32>,
  trees: vec4<f32>,
  anim: vec4<f32>,
};

// A unit gradient for a cell.
fn foliage_gradient(cell: vec2<f32>) -> vec2<f32> {
  let angle: f32 = fract(sin(dot(cell, vec2<f32>(127.1, 311.7))) * 43758.5453) * FOLIAGE_TAU;

  return vec2<f32>(cos(angle), sin(angle));
}

// Gradient noise at a point and its slope there: quintic, so the slope is smooth across cells.
fn foliage_noise(at: vec2<f32>) -> vec3<f32> {
  let cell: vec2<f32> = floor(at);
  let f: vec2<f32> = at - cell;
  let u: vec2<f32> = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  let du: vec2<f32> = 30.0 * f * f * (f * (f - 2.0) + 1.0);
  let ga: vec2<f32> = foliage_gradient(cell);
  let gb: vec2<f32> = foliage_gradient(cell + vec2<f32>(1.0, 0.0));
  let gc: vec2<f32> = foliage_gradient(cell + vec2<f32>(0.0, 1.0));
  let gd: vec2<f32> = foliage_gradient(cell + vec2<f32>(1.0, 1.0));
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

// A waving tuft's vertex move at a place on the ground and a height over the tuft's foot: tossed by the flow field,
// pushed downwind and lifted by its gusts, the shortest and the tallest held stiffer.
fn foliage_grass(at: vec2<f32>, height: f32, setup: FoliageSetup) -> vec3<f32> {
  let limit: f32 = saturate(height * height - 0.01) * saturate(1.0 - height * 0.1);
  let flow: vec3<f32> = foliage_flow((at - setup.anim.xy * setup.grass.x) * 0.018);
  let toss: vec2<f32> = (flow.xy * 2.0 - 1.0) * setup.grass.y;
  let push: vec2<f32> = flow.z * setup.wind.xy * setup.grass.z;

  return vec3<f32>(toss.x + push.x, flow.z * setup.grass.w, toss.y + push.y) * setup.wind.z * limit;
}
