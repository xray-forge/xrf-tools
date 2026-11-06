use std::collections::HashMap;

use xrf_editable_object::EditableMesh;

use crate::object::decompile::geometry::welded_face::WeldedFace;

/// Infers each face's smoothing edge flags from the normals the export gave its corners.
///
/// A CoP SDK mesh keeps one word per face, a bit per edge marking it hard (`EditableMesh::get_hard_edge_flag`), and
/// `CEditableMesh::GenerateVNormals` sums, for each corner, the face normals it can reach around its point through soft
/// edges (`EditableMeshNormals`). Two faces smoothed across an edge so carry one normal at both its ends; a hard edge
/// splits them. So an edge is soft where every face across it agrees with this one at both ends, and hard otherwise, an
/// open edge included. Every face is written a front face, and one with no soft edge as the SDK's flat face.
pub(crate) struct SmoothingInference;

impl SmoothingInference {
  /// The word of a front face with every edge hard.
  const ALL_EDGES_HARD: u32 = 0b111;

  pub(crate) fn infer(faces: &[WeldedFace]) -> Vec<u32> {
    let mut faces_of_point: HashMap<u32, Vec<usize>> = HashMap::new();

    for (index, face) in faces.iter().enumerate() {
      for point in face.points {
        faces_of_point.entry(point).or_default().push(index);
      }
    }

    (0..faces.len())
      .map(|index| {
        let mut flags: u32 = 0;

        for edge in 0..3 {
          if !Self::is_soft(faces, &faces_of_point, index, edge) {
            flags |= EditableMesh::get_hard_edge_flag(0, edge);
          }
        }

        if flags == Self::ALL_EDGES_HARD {
          EditableMesh::FLAT_FACE_SMOOTHING
        } else {
          flags
        }
      })
      .collect()
  }

  fn is_soft(faces: &[WeldedFace], faces_of_point: &HashMap<u32, Vec<usize>>, index: usize, edge: usize) -> bool {
    let face: &WeldedFace = &faces[index];
    let ends: [u32; 2] = [face.points[edge], face.points[(edge + 1) % 3]];
    let mut neighbours = faces_of_point[&ends[0]]
      .iter()
      .filter(|other| **other != index && faces[**other].points.contains(&ends[1]))
      .peekable();

    if neighbours.peek().is_none() {
      return false;
    }

    neighbours.all(|other| {
      ends.iter().all(
        |end| match (face.get_corner_at(*end), faces[*other].get_corner_at(*end)) {
          (Some(own), Some(across)) => own.has_normal_of(across),
          _ => false,
        },
      )
    })
  }
}
