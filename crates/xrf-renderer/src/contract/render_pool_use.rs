use serde::{Deserialize, Serialize};

/// How much of a pool of records a frame used: entries held, and entries it has room for before it grows.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPoolUse {
  pub used: u32,
  pub capacity: u32,
}
