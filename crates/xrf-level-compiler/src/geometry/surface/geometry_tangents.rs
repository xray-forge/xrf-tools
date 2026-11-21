use std::collections::HashMap;

use xrf_geometry::{Mesh, MeshVertex, TangentBasis};
use xrf_math::Vector3d;

use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::surface::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::surface::geometry_basis::GeometryBasis;
use crate::geometry::surface::geometry_normals::GeometryNormals;

/// Each face corner's tangent and binormal as `xrPhase_TangentBasis` leaves them on the faces.
///
/// The mender ([`TangentBasis::mend_in_place`]) gets one vertex per split vertex and base coordinate
/// (`fill_mender_input`), and each corner takes its vertex's directions back quantized ([`GeometryBasis::of`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometryTangents {
  /// Each face of the mesh, in its order, its three corners' bases.
  pub faces: Vec<[GeometryBasis; 3]>,
}

impl GeometryTangents {
  /// The bases of `mesh`'s faces over the vertices `normals` split them into, with `coordinates`.
  pub fn of(mesh: &GeometryMesh, normals: &GeometryNormals, coordinates: &GeometryBaseCoordinates) -> Self {
    let mut vertices: Vec<MeshVertex> = Vec::new();
    let mut numbered: HashMap<(u32, [u32; 2]), u32> = HashMap::new();
    let faces: Vec<[u32; 3]> = normals
      .faces
      .iter()
      .zip(&coordinates.faces)
      .map(|(corners, coordinates)| {
        std::array::from_fn(|corner| {
          let split: u32 = corners[corner];
          let coordinate = coordinates[corner];

          *numbered
            .entry((split, [coordinate.x.to_bits(), coordinate.y.to_bits()]))
            .or_insert_with(|| {
              vertices.push(MeshVertex {
                position: normals.positions[split as usize],
                normal: normals.normals[split as usize],
                tangent: Vector3d::ZERO,
                binormal: Vector3d::ZERO,
                texture_coordinate: coordinate,
                bones: Vec::new(),
              });

              (vertices.len() - 1) as u32
            })
        })
      })
      .collect();
    let mended: Mesh<u32> = TangentBasis::mend_in_place(Mesh { vertices, faces });

    debug_assert_eq!(mended.faces.len(), mesh.faces.len(), "the mender keeps every face");

    Self {
      faces: mended
        .faces
        .iter()
        .map(|face| {
          face.map(|vertex| {
            let vertex: &MeshVertex = &mended.vertices[vertex as usize];

            GeometryBasis::of(&vertex.tangent, &vertex.binormal)
          })
        })
        .collect(),
    }
  }
}
