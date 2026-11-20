use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// A numbered hold that keeps the renderer's GPU started while no viewport draws.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RenderHoldId(pub u32);

impl Display for RenderHoldId {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(formatter)
  }
}
