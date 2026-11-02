#import "common/noise"

// The camera's lens and film the present shows the frame through: the lens's natural falloff towards the corners, its
// glass splitting colour towards the edges, and film grain. The models are FidelityFX Lens 1.1's (AMD, MIT), written
// anew: a cos⁴ falloff, transverse aberration by Cauchy's equation, grains on a hexagonal grid.

// Cauchy's equation for K5 crown glass, `n = A + B / λ²` with λ in micrometres, at the hydrogen C line (red), the
// helium d line (green) and the hydrogen F line (blue).
const LENS_CAUCHY_A: f32 = 1.5220;
const LENS_CAUCHY_B: f32 = 0.00459;
const LENS_RED: f32 = 0.6563;
const LENS_GREEN: f32 = 0.5876;
const LENS_BLUE: f32 = 0.4861;

// How far apart red and blue land at the frame's corner at full strength, as a share of the way from its centre.
const LENS_ABERRATION_SPREAD: f32 = 0.008;

// The glass's index at a wavelength.
fn lens_index(wavelength: f32) -> f32 {
  return LENS_CAUCHY_A + LENS_CAUCHY_B / (wavelength * wavelength);
}

// How much more red, green and blue are magnified than green, at `strength` from nothing to one: each channel's
// index against green's over the spread from red to blue, so the split keeps the glass's proportions.
fn lens_magnifications(strength: f32) -> vec3<f32> {
  let green: f32 = lens_index(LENS_GREEN);
  let spread: f32 = lens_index(LENS_BLUE) - lens_index(LENS_RED);
  let shares: vec3<f32> = vec3<f32>(lens_index(LENS_RED) - green, 0.0, lens_index(LENS_BLUE) - green) / spread;

  return 1.0 + shares * strength * LENS_ABERRATION_SPREAD;
}

// The image at `uv` with each channel read where the lens bends it, magnified about the frame's centre.
fn lens_aberrated(image: texture_2d<f32>, image_sampler: sampler, uv: vec2<f32>, strength: f32) -> vec3<f32> {
  let magnifications: vec3<f32> = lens_magnifications(strength);
  let from_centre: vec2<f32> = uv - 0.5;

  return vec3<f32>(
    textureSampleLevel(image, image_sampler, 0.5 + from_centre * magnifications.r, 0.0).r,
    textureSampleLevel(image, image_sampler, uv, 0.0).g,
    textureSampleLevel(image, image_sampler, 0.5 + from_centre * magnifications.b, 0.0).b,
  );
}

// The light reaching the film at `uv` through a lens whose half view spans `tangents` across and up: cos⁴ of the
// ray's angle off the axis, the falloff a wide lens shows at its corners, kept at `strength` from none to all of it.
fn lens_vignette(uv: vec2<f32>, tangents: vec2<f32>, strength: f32) -> f32 {
  let ray: vec2<f32> = (uv * 2.0 - 1.0) * tangents;
  // cos²θ is `1 / (1 + tan²θ)`.
  let cosine_squared: f32 = 1.0 / (1.0 + dot(ray, ray));

  return mix(1.0, cosine_squared * cosine_squared, saturate(strength));
}

// The grain over a pixel, from minus one to one: the pixel's grain on a hexagonal grid of grains `size` pixels across,
// its value its own at `frame`, fading to nothing towards the grain's edge.
fn lens_grain(pixel: vec2<f32>, size: f32, frame: u32) -> f32 {
  // Two rectangular lattices offset by half a cell; the nearer centre of the two is the hexagon's.
  let cell: vec2<f32> = vec2<f32>(1.0, 1.7320508);
  let point: vec2<f32> = pixel / max(size, 0.5);
  let first: vec2<f32> = point - cell * floor(point / cell) - cell * 0.5;
  let shifted: vec2<f32> = point - cell * 0.5;
  let second: vec2<f32> = shifted - cell * floor(shifted / cell) - cell * 0.5;
  let offset: vec2<f32> = select(second, first, dot(first, first) < dot(second, second));
  // The centres stand on halves across and on half rows up, which doubled are whole.
  let centre: vec2<i32> = vec2<i32>(round((point - offset) * vec2<f32>(2.0, 2.0 / cell.y)));
  let hashed: vec3<u32> = pcg3d(vec3<u32>(bitcast<vec2<u32>>(centre), frame));
  // Two uniform values summed lean towards the middle, as grain does.
  let value: f32 = (f32(hashed.x >> 8u) + f32(hashed.y >> 8u)) / 16777216.0 - 1.0;

  return value * (1.0 - smoothstep(0.3, 0.5, length(offset)));
}

// How much of the grain shows over a displayed brightness: most in the mid tones, less in the shadows and highlights.
fn lens_grain_response(luma: f32) -> f32 {
  let shown: f32 = saturate(luma);

  return mix(0.35, 1.0, 4.0 * shown * (1.0 - shown));
}
