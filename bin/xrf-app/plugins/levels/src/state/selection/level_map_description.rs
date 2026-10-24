use serde::Serialize;

/// Where the PDA lays the level's map over it, as the level's `level.ltx` places it in `[level_map]`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMapDescription {
  /// `bound_rect`'s first corner, the least `x` and `z` in engine space.
  pub min_x: f32,
  pub min_z: f32,
  /// Its second corner, the greatest `x` and `z`.
  pub max_x: f32,
  pub max_z: f32,
  /// The map's texture, as `texture` names it, where it names one.
  pub texture: Option<String>,
}
