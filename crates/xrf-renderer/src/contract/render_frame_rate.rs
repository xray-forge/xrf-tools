use serde::{Deserialize, Serialize};

/// How often a viewport's frames are drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RenderFrameRate {
  /// At most this many a second, each presented at a refresh of the display, which a faster limit stops at.
  Limited { frames_per_second: u32 },
  /// As fast as a frame can be drawn, presented as soon as it is, for measuring.
  Unlimited,
}

impl RenderFrameRate {
  /// The rate a viewport starts at.
  pub const DEFAULT_FRAMES_PER_SECOND: u32 = 60;

  /// Whether frames wait for a refresh of the display to be presented.
  pub fn is_vsync(self) -> bool {
    matches!(self, Self::Limited { .. })
  }

  /// The least time between two frames, or none for no limit.
  pub fn get_interval(self) -> Option<std::time::Duration> {
    match self {
      Self::Limited { frames_per_second } => Some(std::time::Duration::from_secs_f64(
        1.0 / f64::from(frames_per_second.max(1)),
      )),
      Self::Unlimited => None,
    }
  }
}

impl Default for RenderFrameRate {
  fn default() -> Self {
    Self::Limited {
      frames_per_second: Self::DEFAULT_FRAMES_PER_SECOND,
    }
  }
}
