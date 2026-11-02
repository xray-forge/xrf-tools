use serde::{Deserialize, Serialize};

/// One resource a frame made for itself, as the frame graph placed it: its label, the pooled texture or buffer of its
/// kind it shares, the passes it lives between, and its bytes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphTransient {
  pub label: String,
  pub is_texture: bool,
  pub ordinal: u32,
  pub first: u32,
  pub last: u32,
  pub bytes: u64,
}
