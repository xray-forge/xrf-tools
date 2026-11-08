use std::collections::HashMap;

use crate::object::compile::geometry::export_vertex::ExportVertex;

/// One split cut into children an OGF's 16-bit indices reach.
///
/// The skeleton export keeps a split whole and its `VPack` overflows silently past 65,535 vertices; the static export
/// starts a new part where `VPack` reaches the bound, losing the face it was packing, and its tangent pass can then
/// carry a part past it again. High-poly models other tools exported spread one surface over several children of one
/// texture and shader, which a decompile welds back into one. The engine draws any number of children alike, so a split
/// past [`Self::MAX_VERTICES`] once its tangents are known is cut, its faces taken in order into one child until the
/// next would carry it past the bound, its vertices numbered by first use in each. A split within the bound comes back
/// as one child, as it was.
pub(crate) struct SplitPartition;

impl SplitPartition {
  /// Vertices one child holds at most: `VPack` keeps an index below `u16::MAX`, the export's own bound.
  pub(crate) const MAX_VERTICES: usize = u16::MAX as usize;

  /// The children `vertices` and `faces` cut into, each its vertices and its faces over them.
  pub(crate) fn cut(vertices: Vec<ExportVertex>, faces: &[[u32; 3]]) -> Vec<(Vec<ExportVertex>, Vec<[u16; 3]>)> {
    Self::cut_at(vertices, faces, Self::MAX_VERTICES)
  }

  /// [`Self::cut`] at a bound of `max_vertices`, which tests set low, and never past [`Self::MAX_VERTICES`].
  pub(crate) fn cut_at(
    vertices: Vec<ExportVertex>,
    faces: &[[u32; 3]],
    max_vertices: usize,
  ) -> Vec<(Vec<ExportVertex>, Vec<[u16; 3]>)> {
    let max_vertices: usize = max_vertices.min(Self::MAX_VERTICES);

    // Every vertex is one a face uses, so a split within the bound keeps its numbering, narrowed.
    if vertices.len() <= max_vertices {
      let faces: Vec<[u16; 3]> = faces.iter().map(|face| face.map(|index| index as u16)).collect();

      return vec![(vertices, faces)];
    }

    let mut children: Vec<(Vec<ExportVertex>, Vec<[u16; 3]>)> = Vec::new();
    let mut child_vertices: Vec<ExportVertex> = Vec::new();
    let mut child_faces: Vec<[u16; 3]> = Vec::new();
    let mut numbers: HashMap<u32, u16> = HashMap::new();

    for face in faces {
      // A face's three vertices are distinct, the packer having dropped every degenerate one.
      let fresh: usize = face.iter().filter(|index| !numbers.contains_key(index)).count();

      if child_vertices.len() + fresh > max_vertices {
        children.push((std::mem::take(&mut child_vertices), std::mem::take(&mut child_faces)));
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
      children.push((child_vertices, child_faces));
    }

    children
  }
}
