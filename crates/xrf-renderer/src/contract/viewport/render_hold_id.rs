use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// One hold on the renderer, which keeps its GPU started while no viewport draws, numbered as it is taken.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RenderHoldId(pub u32);

impl Display for RenderHoldId {
  /// Its number, as a log line names the hold.
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(formatter)
  }
}
