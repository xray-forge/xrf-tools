use serde::Serialize;

/// One field of a spawned entity's data as the scene saves it, `[object_N_spawndata]`: the packet's values numbered in
/// the order they are written, its custom data among them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneSpawnField {
  pub key: String,
  /// The value as the scene stores it.
  pub value: String,
}
