use std::time::Duration;

use serde::{Deserialize, Serialize};

/// How often a viewport's frames are drawn and how they are presented.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderFrameRate {
  /// Frames a second drawn at most, or none for no cap: presented at the display's refresh while `is_vsync`, and as
  /// fast as a frame is drawn otherwise.
  pub limit: Option<u32>,
  /// Whether a frame waits for a refresh of the display to be presented, which a faster limit stops at.
  pub is_vsync: bool,
}

impl RenderFrameRate {
  /// The least time between two frames, or none for no cap; a cap of zero is none.
  pub fn get_interval(self) -> Option<Duration> {
    self
      .limit
      .filter(|limit| *limit > 0)
      .map(|limit| Duration::from_secs_f64(1.0 / f64::from(limit)))
  }
}

impl Default for RenderFrameRate {
  /// The display's own refresh rate: no cap, presented at each refresh.
  fn default() -> Self {
    Self {
      limit: None,
      is_vsync: true,
    }
  }
}
