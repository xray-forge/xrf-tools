// The rings the enhanced drops leave where they land (`scene/level/rain_rings.rs`): a tile of the level seen from
// overhead, wrapping, which the drops' columns repeat over, each texel the tilt the rings give the water in renderer
// `x` and `z`, then that tilt's square, so a level reduced from many rings keeps the tilt they lost to each other as
// its variance. Inside the drops' square each ring is a drop seen landing; past it the tile's rings repeat, as the
// rain past the square would ripple.

// The share of the rings' reach they ripple at their full strength over, fading out over the rest.
const RINGS_FULL: f32 = 0.8;

// The rings at a place: their tilt, its variance their filtering folded away, and how strongly they show there.
struct RainRings {
  tilt: vec2<f32>,
  variance: f32,
  share: f32,
};

// The rings at a `place` in renderer `x` and `z`, as `setup` lays them out: a tile its `x` metres a side, none where
// no drops draw rings, reaching its `y` metres from the `eye` across the ground, however high over it the eye stands.
// Filtered evenly over the widest of its gradients `across` and `down`, so rings far off soften into the variance
// rather than break into grain along a grazing look.
fn read_rain_rings(rings: texture_2d<f32>, rings_sampler: sampler, setup: vec2<f32>, eye: vec2<f32>,
  place: vec2<f32>, across: vec2<f32>, down: vec2<f32>) -> RainRings {
  let tile: f32 = setup.x;
  let distance: f32 = length(place - eye);

  if (tile <= 0.0 || distance >= setup.y) {
    return RainRings(vec2<f32>(0.0), 0.0, 0.0);
  }

  let footprint: f32 = max(length(across), length(down)) / tile;
  let texel: vec4<f32> = textureSampleGrad(rings, rings_sampler, place / tile, vec2<f32>(footprint, 0.0),
    vec2<f32>(0.0, footprint));

  return RainRings(texel.xy, max(texel.z - dot(texel.xy, texel.xy), 0.0),
    smoothstep(setup.y, setup.y * RINGS_FULL, distance));
}
