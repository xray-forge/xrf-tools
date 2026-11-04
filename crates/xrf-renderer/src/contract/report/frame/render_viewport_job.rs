use serde::{Deserialize, Serialize};

/// The long job a viewport runs, one at a time over every viewport: its frames recorded, a burst it draws, or a level
/// map or cubemap it captures in place of its own frames.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderViewportJob {
  Recording,
  Burst,
  Capture,
}
