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
