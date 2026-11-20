use serde::Serialize;

/// A level the game lists, as a front-end offers it for a build.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnListedLevel {
  /// Its directory name, as a request names it.
  pub name: String,
  /// The name a graph point names it by in a connection.
  pub caption: String,
  pub id: u8,
  /// Whether its directory holds `level.ai`, `level.spawn` and `level.game`.
  pub is_complete: bool,
}
