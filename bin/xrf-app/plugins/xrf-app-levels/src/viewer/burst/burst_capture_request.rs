use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderBurst, RenderViewportId};

/// A burst asked of a viewport, the name of the folder of the level viewer's bursts it is written into, and whether it
/// records the sound the viewport's camera hears at the burst's times as `sound.wav` beside its frames.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct BurstCaptureRequest {
  pub viewport: RenderViewportId,
  pub burst: RenderBurst,
  pub name: String,
  pub has_sound: bool,
}
