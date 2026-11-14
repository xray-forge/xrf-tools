use std::collections::HashMap;

use xrf_geometry::{Mesh, MeshVertex, TangentBasis};
use xrf_level::LevelVertexQuantizer;
use xrf_math::Vector3d;

use crate::geometry::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::geometry_basis::GeometryBasis;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_normals::GeometryNormals;

/// Each face corner's tangent and binormal as xrLC's tangent-basis pass leaves them on the faces
/// (`xrPhase_TangentBasis`, `xrLC/xrPhase_TangentBasis.cpp:97`).
///
/// Every face in order gives the mender a vertex per corner: its split vertex's place and normal with the corner's base
/// coordinate, one vertex per split vertex and coordinate (`fill_mender_input`). The mender runs with the arguments
/// both exports give it ([`TangentBasis::mend_in_place`]), and each corner takes its vertex's directions back
/// (`set_face`), normalized safely and quantized to a byte a component (`base_basis::set`).
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

            GeometryBasis {
              tangent: Self::quantize(&vertex.tangent),
              binormal: Self::quantize(&vertex.binormal),
            }
          })
        })
        .collect(),
    }
  }

  /// `base_basis::set`: the direction normalized where it has a length, then a byte a component as `u8_vec4` takes it.
  fn quantize(direction: &Vector3d) -> [u8; 3] {
    let length_squared: f32 = direction.x * direction.x + direction.y * direction.y + direction.z * direction.z;
    let normalized: Vector3d = if length_squared > f32::MIN_POSITIVE {
      let scale: f32 = (1.0 / length_squared).sqrt();

      Vector3d::new(direction.x * scale, direction.y * scale, direction.z * scale)
    } else {
      *direction
    };
    let [z, y, x, _] = LevelVertexQuantizer::get_direction(&normalized, 0);

    [x, y, z]
  }
}
