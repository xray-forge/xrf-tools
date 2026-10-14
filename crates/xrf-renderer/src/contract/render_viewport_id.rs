use serde::{Deserialize, Serialize};
use xrf_renderer_core::GraphOwner;

/// One viewport of the renderer, numbered by the renderer as it is attached.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RenderViewportId(pub u32);

impl From<RenderViewportId> for GraphOwner {
  /// A viewport's passes, which every number keeps apart from the frame's own.
  fn from(id: RenderViewportId) -> Self {
    Self::Part(id.0)
  }
}
