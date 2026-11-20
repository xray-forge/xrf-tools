use glam::{DVec2, DVec3};

/// One rendered triangle of a library object: its mesh, by its place in the object's meshes, its texture, by its place
/// in the library's texture table, its corners in the object's space, and their texture coordinates where its mesh maps
/// them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelLibraryTriangle {
  pub mesh: usize,
  pub texture: u32,
  pub points: [DVec3; 3],
  pub uvs: Option<[DVec2; 3]>,
  /// Whether its surface is two-sided, which the Build writes as the triangle and its mirror.
  pub is_two_sided: bool,
  pub area: f64,
}

impl LevelLibraryTriangle {
  /// The area of the triangle `points` spans.
  pub fn get_area(points: &[DVec3; 3]) -> f64 {
    (points[1] - points[0]).cross(points[2] - points[0]).length() * 0.5
  }

  /// Its edge lengths, shortest first.
  pub fn get_edges(points: &[DVec3; 3]) -> [f64; 3] {
    let mut edges: [f64; 3] = [
      points[0].distance(points[1]),
      points[1].distance(points[2]),
      points[2].distance(points[0]),
    ];

    edges.sort_unstable_by(f64::total_cmp);

    edges
  }
}
