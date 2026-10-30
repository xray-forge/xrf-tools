// The noise the passes that turn their samples per pixel and per frame share.

// The R2 sequence's steps (Roberts, 2018), the plastic number's reciprocal and its square, which spread a pixel's
// samples apart and turn its noise from one frame to the next, as `NoiseTurn::R2_STEPS`.
const R2_STEPS: vec2<f32> = vec2<f32>(0.7548777, 0.5698403);

// Interleaved gradient noise (Jimenez, "Next Generation Post Processing in Call of Duty: Advanced Warfare", SIGGRAPH
// 2014): one value a pixel, from zero to one, neighbours far apart in value.
fn gradient_noise(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

// Two interleaved gradient noises a pixel, the second read about the swapped pixel so the two do not move together.
fn gradient_noise_2(pixel: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(gradient_noise(pixel), gradient_noise(pixel.yx + vec2<f32>(17.0, 59.0)));
}

// Two values from nothing to one a point, each its own (Hoskins, "Hash without Sine"): what a cell or a pixel draws
// apart from its neighbours.
fn hash22(point: vec2<f32>) -> vec2<f32> {
  var p: vec3<f32> = fract(vec3<f32>(point.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));

  p += dot(p, p.yzx + 33.33);

  return fract((p.xx + p.yz) * p.zy);
}

// Three well-mixed words from three (Jarzynski and Olano, "Hash Functions for GPU Rendering", 2020): a cell's own
// values at a frame, none of them repeating along any axis.
fn pcg3d(seed: vec3<u32>) -> vec3<u32> {
  var v: vec3<u32> = seed * 1664525u + 1013904223u;

  v.x += v.y * v.z;
  v.y += v.z * v.x;
  v.z += v.x * v.y;
  v ^= v >> vec3<u32>(16u);
  v.x += v.y * v.z;
  v.y += v.z * v.x;
  v.z += v.x * v.y;

  return v;
}
