use serde::Serialize;

/// How much sky and light reach one spawned object from each way, as the game estimates it for a dynamic object.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnObjectHemi {
  /// The object, by its place among the level's spawned objects.
  pub index: u32,
  /// The faces toward `+x +y +z`, then toward `-x -y -z`, in renderer space.
  pub cube: [f32; 6],
  /// Its scalar sky share, `hemi_value`, which a forward-drawn model is lit by.
  pub sky: f32,
}
