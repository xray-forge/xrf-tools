use xrf_math::{Aabb, Vector3d};

use crate::geometry::geometry_face::GeometryFace;

/// A level's static geometry as xrLC works on it before it splits it by material: the vertices of `build.prj`, welded,
/// and its faces over them. Its steps run in xrLC's order (`CBuild::Run`, `xrLC/Build.cpp:158`): [`Self::load`],
/// [`Self::optimize`] (`PreOptimize`), [`Self::tessellate`] (the first pass of `xrPhase_AdaptiveHT`), then
/// `GeometryNormals::of` (`CalcNormals`).
///
/// Order is kept as xrLC keeps it, since later steps read it: a face removed leaves the others in their order, a face
/// added goes after them all.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryMesh {
  pub vertices: Vec<Vector3d>,
  pub faces: Vec<GeometryFace>,
  /// The box around every vertex `build.prj` holds, used or not, `scene_bb`: what the weld lays its grid over.
  pub bounds: Aabb,
}

impl GeometryMesh {
  /// The faces about each vertex, each list in face order: `m_adjacents`, which xrLC sorts by the faces' addresses,
  /// the order it made them in.
  pub fn get_faces_around_vertices(&self) -> Vec<Vec<u32>> {
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
