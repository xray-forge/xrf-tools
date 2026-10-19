use serde::{Deserialize, Serialize};

/// The size a reflection is traced at, of the frame's: half each way, a ray for every four pixels, or full.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderTraceSize {
  #[default]
  Half,
  Full,
}

impl RenderTraceSize {
  /// The frame's pixels a traced pixel stands for each way.
  pub const fn get_ratio(self) -> u32 {
    match self {
      Self::Half => 2,
      Self::Full => 1,
    }
  }
}
