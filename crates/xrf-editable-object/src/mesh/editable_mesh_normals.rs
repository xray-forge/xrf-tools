use std::collections::HashMap;

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

        (&second - &first).cross(&(&third - &second)).normalize_safe()
      })
      .collect();
    let fans: Vec<FanWalk<'_>> = point_faces
      .iter()
      .map(|around| FanWalk::of(&mesh.faces, &flags, around))
      .collect();
    let corners: Vec<[Vector3d; 3]> = mesh
      .faces
      .iter()
      .enumerate()
      .map(|(index, face)| {
        face.corners.map(|corner| {
          let fan: &FanWalk<'_> = &fans[corner.point as usize];
          let start: usize = fan
            .around
            .iter()
            .position(|face| *face as usize == index)
            .expect("a face is around each of its own points");
          let sum: Vector3d = fan.sum_from(start, &faces);
          let length: f32 = sum.length();

          if length > Self::NEGLIGIBLE_LENGTH {
            &sum / length
          } else {
            faces[fan.around[0] as usize]
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

  /// Whether edge `edge` of a face with smoothing word `flags` is soft.
  fn is_soft_edge(flags: u32, edge: usize) -> bool {
    flags & EditableMesh::get_hard_edge_flag(flags, edge) == 0
  }
}

/// The walks around one point, `itterate_adjacents::RecurseTri` over `itterate_adjacents_params_dynamic`: the faces
/// around it, and which of them each crosses to, found once for every corner at the point.
struct FanWalk<'a> {
  /// The faces around the point, as the adjacency lists them.
  around: &'a [u32],
  /// For each place in `around`, the places the walk crosses to from it, in ascending order, its own among them: a
  /// face reaches itself, which is how a corner's own face is summed.
  links: Vec<Vec<usize>>,
}

impl<'a> FanWalk<'a> {
  /// The walks around the point `around` lists the faces of: each pair of faces sharing an edge, both calling it soft
  /// and their back-face bits agreeing, linked, and each face to itself.
  fn of(faces: &[EditableFace], flags: &[u32], around: &'a [u32]) -> Self {
    let mut by_edge: HashMap<[u32; 2], Vec<usize>> = HashMap::new();

    for (place, face) in around.iter().enumerate() {
      for edge in 0..3 {
        by_edge
          .entry(Self::get_edge(&faces[*face as usize], edge))
          .or_default()
          .push(place);
      }
    }

    let links: Vec<Vec<usize>> = (0..around.len())
      .map(|from| {
        let face: &EditableFace = &faces[around[from] as usize];
        let mut linked: Vec<usize> = (0..3)
          .flat_map(|edge| by_edge[&Self::get_edge(face, edge)].iter().copied())
          .filter(|to| Self::is_connected(faces, flags, around[from] as usize, around[*to] as usize))
          .collect();

        linked.sort_unstable();
        linked.dedup();
        linked
      })
      .collect();

    Self { around, links }
  }

  /// The sum of the normals `face_normals` gives the faces reached from the place `start`, walked depth first through
  /// the links in their order, each face summed as it is first reached: the order the SDK's recursion sums them in.
  /// The walk keeps its own stack, as deep as the fan, rather than the thread's.
  fn sum_from(&self, start: usize, face_normals: &[Vector3d]) -> Vector3d {
    let mut is_reached: Vec<bool> = vec![false; self.around.len()];
    let mut sum: Vector3d = Vector3d::ZERO;
    // Each face being walked from, and how far through its links the walk has gone.
    let mut stack: Vec<(usize, usize)> = vec![(start, 0)];

    while let Some(top) = stack.last_mut() {
      let (from, next): (usize, usize) = *top;
      let Some(to) = self.links[from].get(next).copied() else {
        stack.pop();
        continue;
      };

      top.1 += 1;

      if !is_reached[to] {
        is_reached[to] = true;
        sum += &face_normals[self.around[to] as usize];
        stack.push((to, 0));
      }
    }

    sum
  }

  /// Whether the walk crosses from one face to another: the same face, or one sharing an edge both call soft, their
  /// back-face bits agreeing.
  fn is_connected(faces: &[EditableFace], flags: &[u32], from_face: usize, to_face: usize) -> bool {
    let Some((from_edge, to_edge)) = Self::find_shared_edge(&faces[from_face], &faces[to_face]) else {
      return false;
    };
    let (from_flags, to_flags): (u32, u32) = (flags[from_face], flags[to_face]);

    from_face == to_face
      || (from_flags & EditableMesh::BACK_FACE_SMOOTHING_FLAG == to_flags & EditableMesh::BACK_FACE_SMOOTHING_FLAG
        && EditableMeshNormals::is_soft_edge(from_flags, from_edge)
        && EditableMeshNormals::is_soft_edge(to_flags, to_edge))
  }

  /// Edge `index` of `face` by its points, the lesser first.
  fn get_edge(face: &EditableFace, index: usize) -> [u32; 2] {
    let (start, end): (u32, u32) = (face.corners[index].point, face.corners[(index + 1) % 3].point);

    if start <= end { [start, end] } else { [end, start] }
  }

  /// The first edge of `first` that `second` also has, by point, and its index in each (`has_same_edge`).
  fn find_shared_edge(first: &EditableFace, second: &EditableFace) -> Option<(usize, usize)> {
    (0..3).find_map(|own| {
      (0..3)
        .find(|other| Self::get_edge(first, own) == Self::get_edge(second, *other))
        .map(|other| (own, other))
    })
  }
}
