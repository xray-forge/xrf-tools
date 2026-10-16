// The circle's constants every shader shares, so no two modules name them apart.

const PI: f32 = 3.14159265358979;
const HALF_PI: f32 = 1.57079632679490;
const TAU: f32 = 6.28318530717959;

// `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine.
fn cyclic(phase: f32) -> f32 {
  let f: f32 = fract(phase) * 2.8284271 - 1.4142136;

  return f * f - 1.0;
}
