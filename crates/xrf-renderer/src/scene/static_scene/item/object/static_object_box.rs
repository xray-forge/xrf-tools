use glam::{BVec3, Mat4, Vec3};

/// A spawned object's box as it stands: its model's box in its own space, between `min` and `max`, and where the scene
/// stands it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticObjectBox {
  pub transform: Mat4,
  pub min: Vec3,
  pub max: Vec3,
}

impl StaticObjectBox {
  /// The unit cube's twelve edges, by corners numbered `x + 2y + 4z`.
  pub const EDGES: [(usize, usize); 12] = [
    (0, 1),
    (2, 3),
    (4, 5),
    (6, 7),
    (0, 2),
    (1, 3),
    (4, 6),
    (5, 7),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
  ];

  /// Its twelve edges where the scene stands it.
  pub fn to_edges(self) -> impl Iterator<Item = [Vec3; 2]> {
    let corner = move |index: usize| -> Vec3 {
      self.transform.transform_point3(Vec3::select(
        BVec3::new(index & 1 != 0, index & 2 != 0, index & 4 != 0),
        self.max,
        self.min,
      ))
    };

    Self::EDGES.iter().map(move |(from, to)| [corner(*from), corner(*to)])
  }
}
