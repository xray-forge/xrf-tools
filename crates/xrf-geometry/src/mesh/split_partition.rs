use std::collections::HashMap;

use crate::mesh::mesh::Mesh;
use crate::mesh::mesh_vertex::MeshVertex;

/// One split cut into children an OGF's 16-bit indices reach.
///
/// The exports fail at that bound: the skeleton export's `VPack` overflows silently past 65,535 vertices, and the
/// static export starts a new part there, losing the face it was packing. The engine draws any number of children
/// alike, so a split past [`Self::MAX_VERTICES`] once its tangents are known is cut: faces go in order into one child
/// until the next would pass the bound, vertices numbered by first use in each. A split within the bound comes back as
/// one child, unchanged.
pub struct SplitPartition;

impl SplitPartition {
  /// Vertices one child holds at most: `VPack` keeps an index below `u16::MAX`, the export's own bound.
  pub const MAX_VERTICES: usize = u16::MAX as usize;

  /// The children `mesh` cuts into.
  pub fn cut(mesh: Mesh<u32>) -> Vec<Mesh<u16>> {
    Self::cut_at(mesh, Self::MAX_VERTICES)
  }

  /// [`Self::cut`] at a bound of `max_vertices`, which tests set low, and never past [`Self::MAX_VERTICES`].
  pub(crate) fn cut_at(Mesh { vertices, faces }: Mesh<u32>, max_vertices: usize) -> Vec<Mesh<u16>> {
    let max_vertices: usize = max_vertices.min(Self::MAX_VERTICES);

    // Every vertex is one a face uses, so a split within the bound keeps its numbering, narrowed.
    if vertices.len() <= max_vertices {
      let faces: Vec<[u16; 3]> = faces.iter().map(|face| face.map(|index| index as u16)).collect();

      return vec![Mesh { vertices, faces }];
    }

    let mut children: Vec<Mesh<u16>> = Vec::new();
    let mut child_vertices: Vec<MeshVertex> = Vec::new();
    let mut child_faces: Vec<[u16; 3]> = Vec::new();
    let mut numbers: HashMap<u32, u16> = HashMap::new();

    for face in &faces {
      // A face's three vertices are distinct, the packer having dropped every degenerate one.
      let fresh: usize = face.iter().filter(|index| !numbers.contains_key(index)).count();

      if child_vertices.len() + fresh > max_vertices {
        children.push(Mesh {
          vertices: std::mem::take(&mut child_vertices),
          faces: std::mem::take(&mut child_faces),
        });
        numbers.clear();
      }

      let numbered: [u16; 3] = face.map(|index| {
        *numbers.entry(index).or_insert_with(|| {
          child_vertices.push(vertices[index as usize].clone());

          // Below `max_vertices`, itself at most `u16::MAX`.
          (child_vertices.len() - 1) as u16
        })
      });

      child_faces.push(numbered);
    }

    if !child_faces.is_empty() {
      children.push(Mesh {
        vertices: child_vertices,
        faces: child_faces,
      });
    }

    children
  }
}
