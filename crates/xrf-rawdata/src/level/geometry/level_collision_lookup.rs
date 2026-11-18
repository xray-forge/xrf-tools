/// Where a render triangle lies on a level's collision form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelCollisionLookup {
  /// On a collision face of the very same three points, of this game material.
  Exact(u16),
  /// On the collision face its normal through its centre crosses nearest it: the level compiler split the render face and
  /// kept the collision face whole, or moved its points.
  Traced(u16),
  /// On none: the surface was built with a compiler shader of no collision.
  None,
}

impl LevelCollisionLookup {
  /// The game material it lies on, if any.
  pub const fn get_material(self) -> Option<u16> {
    match self {
      Self::Exact(material) | Self::Traced(material) => Some(material),
      Self::None => None,
    }
  }
}
