use std::collections::HashMap;

use crate::object::decompile::geometry::welded_face::WeldedFace;

/// Infers each face's smoothing edge flags from the normals the export gave its corners.
///
/// A CoP SDK mesh keeps one word per face (`face_smoth_flags.h`): bit `e` set makes edge `e`, from corner `e` to
/// corner `(e + 1) % 3`, hard, and `CEditableMesh::GenerateVNormals` sums, for each corner, the face normals it can
/// reach around its point through soft edges. Two faces smoothed across an edge so carry one normal at both its ends;
/// a hard edge splits them. So an edge is soft where every face across it agrees with this one at both ends, and hard
/// otherwise, an open edge included. A face with no soft edge is the SDK's flat face, every bit set.
pub(crate) struct SmoothingInference;

impl SmoothingInference {
  /// The flat face: every edge hard, as the SDK fills a mesh without smoothing data.
  pub(crate) const FLAT: u32 = u32::MAX;

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
            flags |= 1 << edge;
          }
        }

        if flags == 0b111 { Self::FLAT } else { flags }
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
