use serde::{Deserialize, Serialize};

use crate::contract::render_backend::RenderBackend;

/// Whether the renderer can draw with a backend on this machine: the GPU it would use, or why it cannot draw.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderBackendAvailability {
  pub backend: RenderBackend,
  /// The adapter it would draw on; none if it cannot draw.
  pub adapter: Option<String>,
  /// The adapter's driver and version as the API reports them, for example `NVIDIA 617.14`; none if it cannot draw.
  pub driver: Option<String>,
  /// Why it cannot draw; none if it can.
  pub problem: Option<String>,
  /// Whether `XRF_RENDER_BACKEND` names it, so the renderer starts on it instead of the settings' choice.
  pub is_forced: bool,
}
