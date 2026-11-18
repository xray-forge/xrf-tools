use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderBurst, RenderViewportId};

/// A burst asked of a viewport: `name` is its folder among the level viewer's bursts, `has_sound` records what the
/// camera hears as `sound.wav`, and `has_video` writes the frames with that sound as `burst.mp4`.
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
