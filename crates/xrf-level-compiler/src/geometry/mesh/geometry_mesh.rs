use xrf_math::{Aabb, Vector3d};

use crate::geometry::mesh::geometry_face::GeometryFace;

/// A level's static geometry before xrLC splits it by material: `build.prj`'s vertices, welded, and its faces.
///
/// Its steps run in `CBuild::Run`'s order: [`Self::load`], [`Self::optimize`], [`Self::tessellate`], then
/// `GeometryNormals::of`. Later steps read the face order, so it is kept as xrLC keeps it: a removed face leaves the
/// others in order, an added one goes last.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GeometryMesh {
  pub vertices: Vec<Vector3d>,
  pub faces: Vec<GeometryFace>,
  /// The box around every `build.prj` vertex, used or not, `scene_bb`: what the weld lays its grid over.
  pub bounds: Aabb,
}

impl GeometryMesh {
  /// The faces about each vertex in face order: `m_adjacents`, which xrLC sorts by face address, its creation order.
  pub(crate) fn get_faces_around_vertices(&self) -> Vec<Vec<u32>> {
    let mut around: Vec<Vec<u32>> = vec![Vec::new(); self.vertices.len()];

    for (index, face) in self.faces.iter().enumerate() {
      for vertex in face.vertices {
        let faces: &mut Vec<u32> = &mut around[vertex as usize];
        let index: u32 = index as u32;

        // A face holding a vertex twice is listed once, as `prep_add` lists it.
        if faces.last() != Some(&index) {
          faces.push(index);
        }
      }
    }

    around
  }
}
