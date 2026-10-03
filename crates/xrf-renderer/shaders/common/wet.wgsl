// What the wet passes read of the rain, as `pass/wet_uniform.rs` writes it.

struct Wet {
  // `RainDensity.x`.
  density: f32,
  // `timers.x`: seconds the rain has fallen.
  time: f32,
  // One on Anomaly's engine.
  is_extended: f32,
  pad: f32,
  // The cover's centre in `x` and `z`, its half width, and the height it is seen from.
  window: vec4<f32>,
};
