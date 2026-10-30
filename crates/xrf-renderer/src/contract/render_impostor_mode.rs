use serde::{Deserialize, Serialize};

/// Which impostors stand in for a level's distant trees.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderImpostorMode {
  /// The engines' own: each tree's impostor the level compiler baked, eight views around it.
  #[default]
  Engine,
  /// Baked from each tree type's own meshes at the level's opening: a hemi-octahedral impostor, seen from any side and
  /// from above (Brucks's octahedral impostors, Epic's Impostor Baker).
  Baked,
}
