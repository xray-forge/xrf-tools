use serde::{Deserialize, Serialize};

use crate::contract::render_backend::RenderBackend;

/// Whether the renderer can draw with a backend on this machine: the GPU it would draw on, or why it cannot.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderBackendAvailability {
  pub backend: RenderBackend,
  /// The adapter it would draw on, none where it cannot draw.
  pub adapter: Option<String>,
  /// Why it cannot draw, none where it can.
  pub problem: Option<String>,
}
