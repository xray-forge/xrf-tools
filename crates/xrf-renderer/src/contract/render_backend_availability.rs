use serde::{Deserialize, Serialize};

use crate::contract::render_backend::RenderBackend;
use crate::contract::render_gpu_description::RenderGpuDescription;

/// Whether the renderer can draw with a backend on this machine: the GPUs it can use, or why it cannot draw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderBackendAvailability {
  pub backend: RenderBackend,
  /// Every hardware GPU of the backend with the features the renderer needs, the one chosen automatically first; empty
  /// if it cannot draw.
  pub gpus: Vec<RenderGpuDescription>,
  /// Why it cannot draw; none if it can.
  pub problem: Option<String>,
  /// Whether `XRF_RENDER_BACKEND` names it, so the renderer starts on it instead of the settings' choice.
  pub is_forced: bool,
}
