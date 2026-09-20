use serde::Serialize;

/// Byte range of one packed attribute inside a visual's geometry buffer.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSection {
  pub byte_offset: u32,
  pub byte_length: u32,
}
