// The rings the enhanced drops leave where they land (`feature/effects/rain/rings/rain_rings.rs`): a tile of the level seen from
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

// The rings at a `place` in renderer `x` and `z`, as `setup` lays them out: a tile its `x` metres a side, none where
// no drops draw rings, reaching its `y` metres from the `eye` across the ground, however high over it the eye stands.
// Filtered evenly over the widest of its gradients `across` and `down`, so rings far off soften into the variance
// rather than break into grain along a grazing look; and read smoothly (`smooth_rain_rings`) where a texel of their
// first level spans more than a pixel, all of it from two pixels a texel.
fn read_rain_rings(rings: texture_2d<f32>, rings_sampler: sampler, setup: vec2<f32>, eye: vec2<f32>,
  place: vec2<f32>, across: vec2<f32>, down: vec2<f32>) -> RainRings {
  let tile: f32 = setup.x;
  let distance: f32 = length(place - eye);

  if (tile <= 0.0 || distance >= setup.y) {
    return RainRings(vec2<f32>(0.0), 0.0, 0.0);
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

  return RainRings(texel.xy, max(texel.z - dot(texel.xy, texel.xy), 0.0),
    smoothstep(setup.y, setup.y * RINGS_FULL, distance));
}
