use serde::Serialize;

/// Objects of a level's spawn list the typed read refused for one reason, which an unpack wrote verbatim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnVerbatimGroup {
  /// Why the typed read refused them, as each one's record states it.
  pub reason: String,
  pub objects: usize,
}
