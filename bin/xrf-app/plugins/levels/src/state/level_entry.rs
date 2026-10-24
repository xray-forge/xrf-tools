use serde::Serialize;

/// One compiled level the roots hold, as a picker lists it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelEntry {
  /// The name the installation knows the level by, which is its directory under `levels`.
  pub name: String,
  /// Its engine identity, which is what opening it takes.
  pub logical_path: String,
  /// Whether `level.geom` sits beside the bundle; a level without it draws nothing.
  pub has_geometry: bool,
}
