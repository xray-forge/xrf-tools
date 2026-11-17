use serde::Serialize;

/// A level the game lists, as a front-end offers it for a build: by its directory name, and whether that directory
/// holds what the build reads.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnListedLevel {
  /// Its directory name, which a request names it by.
  pub name: String,
  /// The name a graph point names it by in a connection.
  pub caption: String,
  pub id: u8,
  /// Whether its directory holds `level.ai`, `level.spawn` and `level.game`.
  pub is_complete: bool,
}
