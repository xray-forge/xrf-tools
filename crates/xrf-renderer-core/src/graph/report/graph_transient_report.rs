use serde::Serialize;

/// One transient as the compile placed it: the slot of its key it shares, the passes it lives between, its size.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphTransientReport {
  pub label: String,
  pub is_texture: bool,
  pub ordinal: usize,
  pub first: usize,
  pub last: usize,
  pub bytes: u64,
}
