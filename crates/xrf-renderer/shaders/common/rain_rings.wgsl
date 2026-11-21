// The rings the enhanced drops leave where they land (`feature/effects/rain/rings/rain_rings.rs`): a tile of the level
// seen from overhead, wrapping, which the drops' columns repeat over, each texel the tilt the rings give the water in
// renderer `x` and `z`, then that tilt's square, so a level reduced from many rings keeps the tilt they lost to each
// other as its variance, then the square of what landings in water made of it. Inside the drops' square each ring is a
// drop seen landing, and only those landing in water ring; past it the tile's rings repeat, every one of them, as the
// rain past the square would ripple.

// The share of the rings' reach they ripple at their full strength over, fading out over the rest.
const RINGS_FULL: f32 = 0.8;

// Metres in from the edge of the drops' square over which its rings go over to the tile's repeats.
const RINGS_SQUARE_EDGE: f32 = 1.0;

// How the rings are laid out: the tile's metres a side, which the drops' square is across too, none where no drops
// draw rings; the metres from the eye they reach; and where that square stands from the eye, in renderer `x` and `z`.
struct RainRingsSetup {
  tile: f32,
  reach: f32,
  aim: vec2<f32>,
};

// The rings at a place: their tilt, its variance their filtering folded away, how strongly they show there, and how
// much of the place stands in the drops' square, where a ring is a landing seen.
struct RainRings {
  tilt: vec2<f32>,
  variance: f32,
  share: f32,
  seen: f32,
};

// The rings' first level at `uv`, read as a cubic B-spline through four bilinear taps (Sigg and Hadwiger, "Fast
// Third-Order Texture Filtering", GPU Gems 2): a bilinear read's slope breaks at every texel's edge, which a mirror
// bent by the tilt shows as the texels' own squares once a texel spans several pixels; the spline's does not.
fn smooth_rain_rings(rings: texture_2d<f32>, rings_sampler: sampler, uv: vec2<f32>) -> vec4<f32> {
  let size: vec2<f32> = vec2<f32>(textureDimensions(rings));
  let at: vec2<f32> = uv * size - 0.5;
  let base: vec2<f32> = floor(at);
  let f: vec2<f32> = at - base;
  let f2: vec2<f32> = f * f;
  let f3: vec2<f32> = f2 * f;
  let w0: vec2<f32> = (1.0 - 3.0 * f + 3.0 * f2 - f3) / 6.0;
  let w1: vec2<f32> = (4.0 - 6.0 * f2 + 3.0 * f3) / 6.0;
  let w3: vec2<f32> = f3 / 6.0;
  let near: vec2<f32> = w0 + w1;
  let far: vec2<f32> = 1.0 - near;
  let low: vec2<f32> = (base - 0.5 + w1 / near) / size;
  let high: vec2<f32> = (base + 1.5 + w3 / far) / size;

  return (textureSampleLevel(rings, rings_sampler, vec2<f32>(low.x, low.y), 0.0) * near.x +
    textureSampleLevel(rings, rings_sampler, vec2<f32>(high.x, low.y), 0.0) * far.x) * near.y +
    (textureSampleLevel(rings, rings_sampler, vec2<f32>(low.x, high.y), 0.0) * near.x +
    textureSampleLevel(rings, rings_sampler, vec2<f32>(high.x, high.y), 0.0) * far.x) * far.y;
}

// The rings at a `place` in renderer `x` and `z`, as `setup` lays them out, reaching its metres from the `eye` across
// the ground, however high over it the eye stands. Filtered evenly over the widest of its gradients `across` and
// `down`, so rings far off soften into the variance rather than break into grain along a grazing look; and read
// smoothly (`smooth_rain_rings`) where a texel of their first level spans more than a pixel, all of it from two pixels
// a texel. In the drops' square, where each ring is a landing seen, only what landings in water made of the tilt is
// kept: one from the ground beside a puddle would cross it with nothing in the water to have made it. Past the square
// the tile repeats and a ring stands for rain no drop is seen making, so all of it ripples.
fn read_rain_rings(rings: texture_2d<f32>, rings_sampler: sampler, setup: RainRingsSetup, eye: vec2<f32>,
  place: vec2<f32>, across: vec2<f32>, down: vec2<f32>) -> RainRings {
  let tile: f32 = setup.tile;
  let distance: f32 = length(place - eye);

  if (tile <= 0.0 || distance >= setup.reach) {
    return RainRings(vec2<f32>(0.0), 0.0, 0.0, 0.0);
  }

  let footprint: f32 = max(length(across), length(down)) / tile;
  let uv: vec2<f32> = place / tile;
  var texel: vec4<f32> = textureSampleGrad(rings, rings_sampler, uv, vec2<f32>(footprint, 0.0),
    vec2<f32>(0.0, footprint));
  // Pixels a texel of the first level spans, less one: nought where the level is read at its own size or smaller.
  let magnified: f32 = saturate(1.0 / max(footprint * f32(textureDimensions(rings).x), 1e-6) - 1.0);

  if (magnified > 0.0) {
    texel = mix(texel, smooth_rain_rings(rings, rings_sampler, uv), magnified);
  }

  let from_square: vec2<f32> = abs(place - eye - setup.aim);
  let seen: f32 = 1.0 - smoothstep(tile * 0.5 - RINGS_SQUARE_EDGE, tile * 0.5, max(from_square.x, from_square.y));
  let kept: f32 = mix(1.0, saturate(texel.w / max(texel.z, 1e-12)), seen);

  return RainRings(texel.xy * kept, max(texel.z - dot(texel.xy, texel.xy), 0.0) * kept,
    smoothstep(setup.reach, setup.reach * RINGS_FULL, distance), seen);
}
