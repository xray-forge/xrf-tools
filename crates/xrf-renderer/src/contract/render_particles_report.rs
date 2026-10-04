use serde::{Deserialize, Serialize};

/// What a level's particle systems came to over the span reported.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParticlesReport {
  /// Effects playing, each of a group's counted.
  pub effects: u32,
  /// Particles alive in them.
  pub particles: u32,
  /// Effects that took an update on the last frame counted, drawn or scheduled.
  pub simulated: u32,
  /// Effects drawn on the last frame counted.
  pub drawn: u32,
  /// Mean milliseconds a frame spent stepping them, on the render thread's workers.
  pub simulation_time: f32,
}
