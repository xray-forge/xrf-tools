use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderBurst, RenderViewportId};

/// A burst asked of a viewport, and the name of the folder of the level viewer's bursts it is written into.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct BurstCaptureRequest {
  pub viewport: RenderViewportId,
  pub burst: RenderBurst,
  pub name: String,
}
