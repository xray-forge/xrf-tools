use serde::{Deserialize, Serialize};

/// What the renderer holds on the GPU for what it draws.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderMemoryReport {
  /// Bytes of every texture uploaded, shared by every viewport.
  pub textures: u64,
  /// Bytes of this viewport's scene buffers: geometry, clusters, places and the rest that grow with what it shows.
  pub scene: u64,
}
