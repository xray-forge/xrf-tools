use serde::{Deserialize, Serialize};

/// One path that was read more than the others.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayReadTraceHotPath {
  pub path: String,
  pub reads: u64,
  pub bytes: u64,
}
