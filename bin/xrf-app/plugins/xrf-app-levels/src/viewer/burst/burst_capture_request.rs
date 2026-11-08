use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderBurst, RenderViewportId};

/// A burst asked of a viewport, the name of the folder of the level viewer's bursts it is written into, whether it
/// records the sound the viewport's camera hears at the burst's times as `sound.wav` beside its frames, and whether it
/// writes its frames with that sound as `burst.mp4`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct BurstCaptureRequest {
  pub viewport: RenderViewportId,
  pub burst: RenderBurst,
  pub name: String,
  pub has_sound: bool,
  pub has_video: bool,
}
