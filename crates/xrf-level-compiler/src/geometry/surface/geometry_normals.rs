use xrf_math::{EPS_S, Vector3d};

use crate::geometry::mesh::geometry_face::GeometryFace;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;

/// A mesh's vertices split by smoothing and each given its normal, as `CBuild::CalcNormals` does
/// (`calculate_normals::calc_normals`, `xrLC_Light/calculate_normals.h`).
///
/// Each vertex's faces are taken in face order and grouped: a group grows from its first face, depth first, through
/// faces sharing an edge with one already in it where both faces' flags call that edge soft and both are fronts or both
/// backs (`do_connect_faces_by_faces_edge_flags`). Each group gets a vertex of its own at the vertex's place, its
/// normal the sum of its faces' normals in the order the group took them, normalised (`normalFromAdj`). A face's normal
/// is `(v1 - v0) × (v2 - v1)` normalised (`CalcNormal`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GeometryNormals {
  /// Each vertex's place, as the mesh's vertex it was split from.
  pub positions: Vec<Vector3d>,
  pub normals: Vec<Vector3d>,
  /// Each face of the mesh, in its order, over the split vertices.
  pub faces: Vec<[u32; 3]>,
}

impl GeometryNormals {
  /// The vertices of `mesh` split by smoothing, with their normals.
  ///
  /// Vertices are taken in order, and a split vertex's faces stop sharing an edge through it with the faces of its
  /// other groups, so a split decides what later vertices find joined, as in xrLC. Split vertices are numbered past the
  /// mesh's own while the pass runs, as `CreateCopy_NOADJ` appends them, and from 0 in the result.
  pub(crate) fn of(mesh: &GeometryMesh) -> Self {
    let face_normals: Vec<Vector3d> = mesh
      .faces
      .iter()
      .map(|face| Self::get_face_normal(&face.get_corners(&mesh.vertices)))
      .collect();
    let around: Vec<Vec<u32>> = mesh.get_faces_around_vertices();
    let first_split: u32 = mesh.vertices.len() as u32;
    let mut corners: Vec<[u32; 3]> = mesh.faces.iter().map(|face| face.vertices).collect();
    let mut positions: Vec<Vector3d> = Vec::with_capacity(mesh.vertices.len());
    let mut normals: Vec<Vector3d> = Vec::with_capacity(mesh.vertices.len());

    for (vertex, faces) in around.iter().enumerate() {
      let mut remaining: Vec<u32> = faces.clone();

      while !remaining.is_empty() {
        let group: Vec<u32> = Self::get_group(&mesh.faces, &corners, &remaining);
        let split: u32 = first_split + positions.len() as u32;
        let mut sum: Vector3d = Vector3d::ZERO;

        for face in &group {
          sum = &sum + &face_normals[*face as usize];

          for corner in &mut corners[*face as usize] {
            if *corner == vertex as u32 {
              *corner = split;
            }
          }
        }

        positions.push(mesh.vertices[vertex]);
        normals.push(sum.exact_normalize());
        remaining.retain(|face| !group.contains(face));
      }
    }

    Self {
      positions,
      normals,
      faces: corners
        .into_iter()
        .map(|face| face.map(|corner| corner - first_split))
        .collect(),
    }
  }

  /// The group the first of `faces` grows, in the order it takes them (`RecurseTri` from the first face, each step
  /// `add_adjacents`): a face not yet taken joins where it is the face grown from, or shares an edge with it, by the
  /// vertices `corners` holds now, that both call soft, both fronts or both backs.
  fn get_group(all: &[GeometryFace], corners: &[[u32; 3]], faces: &[u32]) -> Vec<u32> {
    let mut is_taken: Vec<bool> = vec![false; faces.len()];
    let mut group: Vec<u32> = Vec::new();
    // Each open call of the recursion: the face it grows from, and the next face it tests.
    let mut calls: Vec<(usize, usize)> = vec![(0, 0)];

    while let Some((start, next)) = calls.last_mut() {
      let (start, test) = (*start, *next);

      if test == faces.len() {
        calls.pop();
        continue;
      }

      *next += 1;

      if is_taken[test] {
        continue;
      }

      let (from, to) = (faces[start] as usize, faces[test] as usize);
      let Some([own, other]) = Self::get_shared_edge(&corners[from], &corners[to]) else {
        continue;
      };
      let (from, to) = (&all[from], &all[to]);

      if start == test || (from.is_back_face() == to.is_back_face() && from.is_soft_edge(own) && to.is_soft_edge(other))
      {
        is_taken[test] = true;
        group.push(faces[test]);
        calls.push((test, 0));
      }
    }

    group
  }

  /// The first edge of `first`, and the edge of `second`, the two faces share by their vertices (`has_same_edge`).
  fn get_shared_edge(first: &[u32; 3], second: &[u32; 3]) -> Option<[usize; 2]> {
    let ends = |face: &[u32; 3], edge: usize| -> [u32; 2] {
      let [a, b] = GeometryFace::EDGES[edge].map(|corner| face[corner]);

      if a > b { [b, a] } else { [a, b] }
    };

    (0..3).find_map(|own| {
      (0..3)
        .find(|other| ends(first, own) == ends(second, *other))
        .map(|other| [own, other])
    })
  }

  /// `Face::CalcNormal`: `(v1 - v0) × (v2 - v1)`, divided by its length and normalised again, or for one shorter than
  /// `EPS_S` normalised in double precision.
  pub(crate) fn get_face_normal([a, b, c]: &[Vector3d; 3]) -> Vector3d {
    let along: Vector3d = b - a;
    let across: Vector3d = c - b;
    let cross: Vector3d = along.cross(&across);
    let length: f32 = (cross.x * cross.x + cross.y * cross.y + cross.z * cross.z).sqrt();

    if length < EPS_S {
      return cross.exact_normalize();
    }

    Vector3d::new(cross.x / length, cross.y / length, cross.z / length).normalize()
  }
}
