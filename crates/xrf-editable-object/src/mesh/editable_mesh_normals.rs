use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::{EditableFace, EditableMesh};

/// A mesh's normals as the SDK's editors compute them, which is what they draw and what the OGF export stores.
///
/// `CEditableMesh::GenerateVNormals` gives each face corner the normalised sum of the face normals reachable around its
/// point, starting from its own face and crossing only edges both faces call soft (`face_smoth_flags.h`), their
/// back-face bits agreeing. The faces are walked depth first in the order the adjacency lists them, so a face is
/// summed once however many paths reach it.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableMeshNormals {
  /// Each face's normal, `(p1 - p0) × (p2 - p1)` at unit length (`Fvector::mknormal`).
  pub faces: Vec<Vector3d>,
  /// Each face's corner normals, in corner order.
  pub corners: Vec<[Vector3d; 3]>,
}

impl EditableMeshNormals {
  /// A summed normal at or below this length takes the first adjacent face's normal instead (`EPS_S`).
  const NEGLIGIBLE_LENGTH: f32 = 1e-7;

  /// The normals of `mesh`.
  ///
  /// # Errors
  ///
  /// Returns an error when a face names a point the mesh does not have, or the mesh holds smoothing words for other
  /// than its faces.
  pub fn of(mesh: &EditableMesh) -> XrfResult<Self> {
    let point_faces: Vec<Vec<u32>> = mesh.get_point_faces()?;
    let flags: Vec<u32> = Self::get_face_flags(mesh)?;
    let faces: Vec<Vector3d> = mesh
      .faces
      .iter()
      .map(|face| {
        let [first, second, third] = face.corners.map(|corner| mesh.points[corner.point as usize]);

        Self::normalize_safe(&(&second - &first).cross(&(&third - &second)))
      })
      .collect();
    let corners: Vec<[Vector3d; 3]> = mesh
      .faces
      .iter()
      .enumerate()
      .map(|(index, face)| {
        face.corners.map(|corner| {
          let around: &[u32] = &point_faces[corner.point as usize];
          let mut walk: FanWalk = FanWalk {
            faces: &mesh.faces,
            flags: &flags,
            face_normals: &faces,
            around,
            is_reached: vec![false; around.len()],
            sum: Vector3d::ZERO,
          };
          let start: usize = around
            .iter()
            .position(|face| *face as usize == index)
            .expect("a face is around each of its own points");

          walk.recurse(start);

          let length: f32 = walk.sum.length();

          if length > Self::NEGLIGIBLE_LENGTH {
            &walk.sum / length
          } else {
            faces[around[0] as usize]
          }
        })
      })
      .collect();

    Ok(Self { faces, corners })
  }

  /// Each face's smoothing word, or what `CEditableMesh::LoadMesh` fills in where the mesh stores none: every edge soft
  /// under the older group mask, every edge hard otherwise.
  fn get_face_flags(mesh: &EditableMesh) -> XrfResult<Vec<u32>> {
    match &mesh.smoothing_groups {
      Some(groups) if groups.len() != mesh.faces.len() => Err(XrfError::new_invalid_error(format!(
        "Mesh '{}' holds {} smoothing words for {} faces",
        mesh.name,
        groups.len(),
        mesh.faces.len()
      ))),
      Some(groups) => Ok(groups.clone()),
      None => Ok(vec![
        if mesh.is_smoothing_group_masked() {
          0
        } else {
          EditableMesh::FLAT_FACE_SMOOTHING
        };
        mesh.faces.len()
      ]),
    }
  }

  /// `Fvector::normalize_safe`: unit length, or the vector untouched when it has no length to divide by.
  fn normalize_safe(vector: &Vector3d) -> Vector3d {
    let length_squared: f32 = vector.length_squared();

    if length_squared > f32::MIN_POSITIVE {
      vector * (1.0 / length_squared).sqrt()
    } else {
      *vector
    }
  }

  /// Whether edge `edge` of a face with smoothing word `flags` is soft.
  fn is_soft_edge(flags: u32, edge: usize) -> bool {
    flags & EditableMesh::get_hard_edge_flag(flags, edge) == 0
  }
}

/// One corner's walk around its point, `itterate_adjacents::RecurseTri` over `itterate_adjacents_params_dynamic`.
struct FanWalk<'a> {
  faces: &'a [EditableFace],
  flags: &'a [u32],
  face_normals: &'a [Vector3d],
  /// The faces around the point, as the adjacency lists them.
  around: &'a [u32],
  is_reached: Vec<bool>,
  sum: Vector3d,
}

impl FanWalk<'_> {
  fn recurse(&mut self, from: usize) {
    for to in 0..self.around.len() {
      if self.reach(from, to) {
        self.recurse(to);
      }
    }
  }

  /// Whether the walk crosses from one face around the point to another not yet reached, summing its normal when it
  /// does. A face reaches itself, which is how the corner's own face is summed.
  fn reach(&mut self, from: usize, to: usize) -> bool {
    if self.is_reached[to] {
      return false;
    }

    let (from_face, to_face): (usize, usize) = (self.around[from] as usize, self.around[to] as usize);
    let Some((from_edge, to_edge)) = Self::find_shared_edge(&self.faces[from_face], &self.faces[to_face]) else {
      return false;
    };
    let (from_flags, to_flags): (u32, u32) = (self.flags[from_face], self.flags[to_face]);
    let is_connected: bool = from_face == to_face
      || (from_flags & EditableMesh::BACK_FACE_SMOOTHING_FLAG == to_flags & EditableMesh::BACK_FACE_SMOOTHING_FLAG
        && EditableMeshNormals::is_soft_edge(from_flags, from_edge)
        && EditableMeshNormals::is_soft_edge(to_flags, to_edge));

    if is_connected {
      self.is_reached[to] = true;
      self.sum += &self.face_normals[to_face];
    }

    is_connected
  }

  /// The first edge of `first` that `second` also has, by point, and its index in each (`has_same_edge`).
  fn find_shared_edge(first: &EditableFace, second: &EditableFace) -> Option<(usize, usize)> {
    let edge = |face: &EditableFace, index: usize| -> [u32; 2] {
      let (start, end): (u32, u32) = (face.corners[index].point, face.corners[(index + 1) % 3].point);

      if start <= end { [start, end] } else { [end, start] }
    };

    (0..3).find_map(|own| {
      (0..3)
        .find(|other| edge(first, own) == edge(second, *other))
        .map(|other| (own, other))
    })
  }
}
