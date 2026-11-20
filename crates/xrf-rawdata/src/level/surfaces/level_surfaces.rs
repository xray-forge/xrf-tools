use serde::{Deserialize, Serialize};

/// How a level's decompile dresses its surfaces: from what the level shows, or with the editor's defaults.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelSurfaces {
  /// Each face's game material from the collision face it lies on, and a compiler shader fitting what the level shows.
  #[default]
  Infer,
  /// Every surface takes `default` for both, as a new surface in the Actor Editor does.
  Defaults,
}
