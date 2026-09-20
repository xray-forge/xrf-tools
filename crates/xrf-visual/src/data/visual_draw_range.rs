use serde::Serialize;

/// The slice of an index buffer that draws one detail level.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualDrawRange {
  pub start: u32,
  pub count: u32,
}
