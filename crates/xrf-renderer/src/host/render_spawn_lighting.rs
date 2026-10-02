/// How the level lights one spawned object, as the game estimates it for a dynamic object standing still.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSpawnLighting {
  /// The object, by its index among the level's spawned objects.
  pub object: u32,
  /// Its hemisphere cube, `+x +y +z -x -y -z`, in renderer space.
  pub cube: [f32; 6],
  /// Its scalar sky share, `hemi_value`.
  pub sky: f32,
}
