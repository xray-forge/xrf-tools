use serde::Serialize;

/// A visual spawned objects name that could not be read, so none of them is drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnModelFailure {
  pub name: String,
  pub reason: String,
}
