use serde::{Deserialize, Serialize};

use crate::contract::render_pass_cost::RenderPassCost;

/// What a viewport's recent frames cost, reported a few times a second while it draws.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderFrameReport {
  /// Frames presented a second over the reported span.
  pub frames_per_second: f32,
  /// Mean milliseconds between presented frames.
  pub frame_time: f32,
  /// Longest milliseconds between two presented frames.
  pub frame_time_max: f32,
  /// Mean milliseconds of the render thread's own work a frame: recording and submitting, not waiting.
  pub cpu_time: f32,
  /// Drawn width, in device pixels.
  pub width: u32,
  /// Drawn height, in device pixels.
  pub height: u32,
  /// The scene's width as rendered, smaller than the drawn one where it is upscaled.
  pub render_width: u32,
  /// And its height.
  pub render_height: u32,
  /// Clusters the last counted frame drew.
  pub clusters: u32,
  /// Triangles they hold, an instanced one counted for every place it stood.
  pub triangles: u32,
  /// The graphics API drawn with.
  pub backend: String,
  /// The GPU drawn on.
  pub adapter: String,
  /// Whether its passes were timed on the GPU over the span.
  pub is_gpu_timed: bool,
  /// What each pass cost on the GPU, in frame order; none while untimed.
  pub passes: Vec<RenderPassCost>,
}
