use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single opening, allocated by its caller before dispatch so it can also be closed while pending.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct DocumentSessionId(Uuid);

impl Display for DocumentSessionId {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(formatter)
  }
}

impl FromStr for DocumentSessionId {
  type Err = uuid::Error;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    value.parse().map(Self)
  }
}

#[cfg(test)]
impl DocumentSessionId {
  pub(crate) fn new() -> Self {
    Self(Uuid::new_v4())
  }
}
