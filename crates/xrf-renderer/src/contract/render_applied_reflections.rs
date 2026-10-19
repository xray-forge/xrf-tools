use serde::{Deserialize, Serialize};

use crate::contract::render_trace_size::RenderTraceSize;

/// The screen-space reflections as traced.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAppliedReflections {
  /// What scales how much of a surface's reflection a traced one replaces, and how much a puddle's coat reflects.
  pub intensity: f32,
  /// The size the rays are traced at, and the pyramid's cells a ray crosses at most.
  pub trace_size: RenderTraceSize,
  pub steps: u32,
}
