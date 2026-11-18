use serde::{Deserialize, Serialize};

/// A viewport's long job, one at a time across all viewports: a recording, a burst, or a map or cubemap capture.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderViewportJob {
  Recording,
  Burst,
  Capture,
}
