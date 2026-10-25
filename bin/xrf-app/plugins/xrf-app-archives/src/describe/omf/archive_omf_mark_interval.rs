use serde::Serialize;

/// One stretch of a motion a mark covers, in seconds from its start.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOmfMarkInterval {
  pub from: f32,
  pub to: f32,
}
