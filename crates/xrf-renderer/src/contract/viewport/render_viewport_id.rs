use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// One viewport of the renderer, numbered by the renderer as it is attached.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RenderViewportId(pub u32);

impl Display for RenderViewportId {
  /// Its number, as a log line names the viewport.
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(formatter)
  }
}
