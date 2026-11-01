use glam::Mat4;

/// Where one spawned object stands as its model, how the level lights it, and the group whose visibility shows it.
#[derive(Clone, Copy, Debug)]
pub struct StaticModelPlace {
  /// The object, by its index among the level's spawned objects.
  pub object: u32,
  /// Its `XFORM`, in renderer space.
  pub transform: Mat4,
  /// Its hemisphere cube, `+x +y +z -x -y -z` in renderer space, and its sky share, where the level's lighting was
  /// estimated for it.
  pub lighting: Option<([f32; 6], f32)>,
  /// The visibility group it is shown in, from one; zero for always.
  pub group: u32,
}
