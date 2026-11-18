use serde::Serialize;

/// The root spawn a decompile read, counted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileRootSpawn {
  pub file: String,
  /// Its objects on the level, which the scene's entities came from; none where it does not hold the level.
  pub objects: usize,
  pub disagreements: usize,
  pub update_packets: usize,
}
