use std::collections::HashSet;

use xrf_math::{Vector2d, Vector3d};

use crate::progressive::progressive_mesh::ProgressiveMesh;
use crate::progressive::progressive_point::ProgressivePoint;

/// A flat grid of `side` by `side` points a unit apart, its texture coordinates following its positions, the height of
/// each point given by `height`.
fn grid(side: u16, height: impl Fn(u16, u16) -> f32) -> (Vec<ProgressivePoint>, Vec<[u16; 3]>) {
  let mut points: Vec<ProgressivePoint> = Vec::new();
  let mut faces: Vec<[u16; 3]> = Vec::new();

  for z in 0..side {
    for x in 0..side {
      points.push(ProgressivePoint {
        position: Vector3d::new(f32::from(x), height(x, z), f32::from(z)),
        coordinate: Vector2d::new(f32::from(x) / 8.0, f32::from(z) / 8.0),
      });
    }
  }

  for z in 0..side - 1 {
    for x in 0..side - 1 {
      let corner: u16 = z * side + x;

      faces.push([corner, corner + side, corner + 1]);
      faces.push([corner + 1, corner + side, corner + side + 1]);
    }
  }

  (points, faces)
}

fn sorted(face: [u16; 3]) -> [u16; 3] {
  let mut face: [u16; 3] = face;

  face.sort_unstable();
  face
}

#[test]
fn test_the_finest_window_is_the_whole_mesh_renumbered() {
  let (points, faces) = grid(8, |_, _| 0.0);
  let mesh: ProgressiveMesh = ProgressiveMesh::build(&points, &faces).expect("a grid is made progressive");
  let finest = mesh.windows[0];
  let start: usize = finest.offset as usize / 3;
  let drawn: HashSet<[u16; 3]> = mesh.faces[start..start + usize::from(finest.triangles)]
    .iter()
    .map(|face| sorted(*face))
    .collect();
  let expected: HashSet<[u16; 3]> = faces
    .iter()
    .map(|face| sorted(face.map(|vertex| mesh.permutation[usize::from(vertex)])))
    .collect();

  assert_eq!(usize::from(finest.triangles), faces.len());
  assert_eq!(usize::from(finest.vertices), points.len());
  assert_eq!(drawn, expected);
}

#[test]
fn test_every_window_draws_only_its_own_vertices_and_coarsens_in_turn() {
  let (points, faces) = grid(8, |_, _| 0.0);
  let mesh: ProgressiveMesh = ProgressiveMesh::build(&points, &faces).expect("a grid is made progressive");

  assert!(mesh.get_collapse_count() > 0);

  for (collapsed, window) in mesh.windows.iter().enumerate() {
    let start: usize = window.offset as usize / 3;

    assert_eq!(usize::from(window.vertices), points.len() - collapsed);
    assert!(
      mesh.faces[start..start + usize::from(window.triangles)]
        .iter()
        .flatten()
        .all(|vertex| *vertex < window.vertices),
      "window {collapsed} names a vertex it has dropped"
    );
  }

  assert!(
    mesh
      .windows
      .windows(2)
      .all(|pair| pair[1].triangles <= pair[0].triangles),
    "each collapse takes triangles away"
  );
  assert!(mesh.windows.last().expect("windows").triangles < mesh.windows[0].triangles);
}

#[test]
fn test_a_vertex_on_the_open_border_never_moves() {
  let side: u16 = 8;
  let (points, faces) = grid(side, |_, _| 0.0);
  let mesh: ProgressiveMesh = ProgressiveMesh::build(&points, &faces).expect("a grid is made progressive");
  let coarsest: u16 = mesh.windows.last().expect("windows").vertices;

  for z in 0..side {
    for x in 0..side {
      if x == 0 || z == 0 || x == side - 1 || z == side - 1 {
        assert!(
          mesh.permutation[usize::from(z * side + x)] < coarsest,
          "border point ({x}, {z}) is kept"
        );
      }
    }
  }
}

#[test]
fn test_a_flat_point_goes_before_a_raised_one() {
  let (points, faces) = grid(8, |x, z| if (x, z) == (3, 3) { 2.0 } else { 0.0 });
  let mesh: ProgressiveMesh = ProgressiveMesh::build(&points, &faces).expect("a grid is made progressive");
  let first_binned: u16 = (points.len() - 1) as u16;

  assert_ne!(
    mesh.permutation[3 * 8 + 3],
    first_binned,
    "the costly point is not collapsed first"
  );
}

#[test]
fn test_refuses_a_mesh_of_a_repeated_face() {
  let (points, mut faces) = grid(4, |_, _| 0.0);

  faces.push([faces[0][0], faces[0][2], faces[0][1]]);

  assert!(
    ProgressiveMesh::build(&points, &faces).is_none(),
    "a face repeated, turned over"
  );
}

#[test]
fn test_a_mesh_with_nothing_to_collapse_has_one_window() {
  let (points, faces) = grid(2, |_, _| 0.0);
  let mesh: ProgressiveMesh = ProgressiveMesh::build(&points, &faces).expect("a quad has its one window");

  assert_eq!(mesh.get_collapse_count(), 0, "a quad's corners all lie on its border");
  assert_eq!(mesh.windows.len(), 1);
  assert_eq!(usize::from(mesh.windows[0].triangles), faces.len());
  assert_eq!(mesh.faces.len(), faces.len());
}
