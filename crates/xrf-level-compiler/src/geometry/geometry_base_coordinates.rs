use xrf_math::{EPS, Vector2d};

use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_normals::GeometryNormals;

/// Each face's base texture coordinates as the tangent-basis pass leaves them (`fill_mender_input`,
/// `xrLC/xrPhase_TangentBasis.cpp:59`), which every later phase reads.
///
/// Faces are taken in order and each corner registered at its split vertex: a corner takes the coordinate of the first
/// one already registered there within `EPS` on both axes (`find_same_vertex`, `fsimilar`), or registers its own. So
/// faces meeting at a vertex with coordinates a rounding apart come out with the same one.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryBaseCoordinates {
  /// Each face of the mesh, in its order, its three corners' coordinates.
  pub faces: Vec<[Vector2d; 3]>,
}

impl GeometryBaseCoordinates {
  /// The base coordinates of `mesh`'s faces over the vertices `normals` split them into.
  pub fn of(mesh: &GeometryMesh, normals: &GeometryNormals) -> Self {
    let mut registered: Vec<Vec<Vector2d>> = vec![Vec::new(); normals.positions.len()];
    let faces: Vec<[Vector2d; 3]> = mesh
      .faces
      .iter()
      .zip(&normals.faces)
      .map(|(face, corners)| {
        std::array::from_fn(|corner| {
          let coordinate: Vector2d = face.texture_coordinates[corner];
          let at: &mut Vec<Vector2d> = &mut registered[corners[corner] as usize];

          match at.iter().find(|held| Self::is_similar(held, &coordinate)) {
            Some(held) => *held,
            None => {
              at.push(coordinate);
              coordinate
            }
          }
        })
      })
      .collect();

    Self { faces }
  }

  /// `fsimilar` at its default on both axes: strictly within `EPS`.
  fn is_similar(first: &Vector2d, second: &Vector2d) -> bool {
    (first.x - second.x).abs() < EPS && (first.y - second.y).abs() < EPS
  }
}
