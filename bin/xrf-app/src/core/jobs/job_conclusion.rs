use serde::Serialize;

/// How a job that is no longer running ended.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobConclusion {
  Completed,
  Cancelled,
  Failed,
}
