use serde::Serialize;

use crate::parts::LevelMapArea;

/// Where the PDA lays the level's map over it, as the level's `level.ltx` places it in `[level_map]`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMapDescription {
  /// The ground the map covers, as `bound_rect` places it.
  pub area: LevelMapArea,
  /// The map's texture, as `texture` names it, where it names one.
  pub texture: Option<String>,
}
