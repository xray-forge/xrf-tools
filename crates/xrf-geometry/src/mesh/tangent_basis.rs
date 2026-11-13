use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};

use xrf_error::{XrfError, XrfResult};
use xrf_math::{Vector2d, Vector3d};

use crate::mesh::mesh::Mesh;
use crate::mesh::mesh_vertex::MeshVertex;

/// Gives a split's vertices tangents and binormals as both exports' tangent pass does, splitting a vertex where the
/// faces around it disagree, then orders and shifts the vertices as the export leaves them.
///
/// `CExportSkeleton::SSplit::CalculateTB` and the static export's `CObjectOGFCollectorPacked::CalculateTB` run NVIDIA's
/// MeshMender (2003) over the split with the same arguments, keeping the normals and the existing vertex splits, at a
/// smoothing cosine of 0.5 for tangents and binormals. The static export then writes no tangents, but its vertices are
/// the ones the pass split and ordered. Each face gets a tangent and a
/// binormal from how its texture coordinates run across it (Lengyel's gradients). Then, position by position in `x`, `y`,
/// `z` order, the faces at a position are grouped: a face joins the group of the first or the second face sharing an
/// edge with it, by vertex, when their tangents are within the cosine, or starts a group of its own. A vertex a second
/// group uses is split off for it, and each group's vertices take its normalised sum; binormals are grouped the same way
/// after. A last step meant to square each tangent and binormal to the normal only rescales them: the mender's
/// `Vec3Normalize` multiplies the vector it writes into, here the smoothed tangent, by the inverse length of the one it
/// measures, the squared tangent, so the smoothed direction stays and grows slightly longer. Vanilla's OGFs carry exactly
/// that (binormals of length 1.0003, off square to the normal), so it is kept; a basis that collapses is replaced by one
/// built from the normal. The pass then keeps the vertices faces use, in the order the faces first reach them
/// (`t_remove_isolated_verts`), and moves the split's texture coordinates by whole tiles towards the origin
/// (`OptimizeTextureCoordinates`).
///
/// One difference is deliberate: where the mender divides by a zero length into infinity or NaN and ships it, a zero
/// length here leaves a zero vector, and the collapsed-basis fallback then builds a valid one.
pub struct TangentBasis {
  vertices: Vec<MeshVertex>,
  triangles: Vec<MendedFace>,
}

/// One face being mended: its vertices, which splitting renumbers, its own tangent and binormal, and where the grouping
/// at the current position has put it.
struct MendedFace {
  indices: [usize; 3],
  tangent: Vector3d,
  binormal: Vector3d,
  group: Option<usize>,
  is_handled: bool,
}

/// Which of a vertex's directions a grouping pass smooths.
#[derive(Clone, Copy)]
enum Direction {
  Tangent,
  Binormal,
}

/// A position as the mender's map orders and compares it: exactly, `x` then `y` then `z`, `-0` equal to `0`.
#[derive(Clone, Copy, Debug)]
struct PositionKey([f32; 3]);

impl PositionKey {
  fn of(position: &Vector3d) -> Self {
    // Adding zero turns `-0` into `0`, which `==` already takes them for.
    Self(position.to_array().map(|component| component + 0.0))
  }
}

impl PartialEq for PositionKey {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}

impl Eq for PositionKey {}

impl PartialOrd for PositionKey {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for PositionKey {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .0
      .iter()
      .zip(other.0)
      .map(|(own, other)| own.total_cmp(&other))
      .find(|ordering| *ordering != Ordering::Equal)
      .unwrap_or(Ordering::Equal)
  }
}

impl TangentBasis {
  /// How close two faces' tangents, or binormals, must lie to smooth together: the cosine of their angle.
  const SMOOTHING_COSINE: f32 = 0.5;

  /// A texture mapping across a face whose determinant is this small is taken as degenerate, keeping its sign.
  const DEGENERATE_MAPPING: f32 = 1e-4;

  /// Below this length a final tangent or binormal is no direction at all.
  const COLLAPSED_LENGTH: f32 = 1e-3;

  /// `mesh` with tangents, binormals, the vertices split where faces disagree, ordered by first use, and texture
  /// coordinates moved by whole tiles.
  ///
  /// # Errors
  ///
  /// Returns an error when the split outgrows 32-bit indices.
  pub fn mend(Mesh { vertices, faces }: Mesh<u32>) -> XrfResult<Mesh<u32>> {
    let mut basis: Self = Self {
      triangles: faces
        .iter()
        .map(|face| {
          let indices: [usize; 3] = face.map(|index| index as usize);
          let (tangent, binormal) = Self::get_gradients(indices.map(|index| &vertices[index]));

          MendedFace {
            indices,
            tangent,
            binormal,
            group: None,
            is_handled: false,
          }
        })
        .collect(),
      vertices,
    };
    let mut triangles_at: BTreeMap<PositionKey, Vec<usize>> = BTreeMap::new();

    for (triangle, face) in basis.triangles.iter().enumerate() {
      for index in face.indices {
        triangles_at
          .entry(PositionKey::of(&basis.vertices[index].position))
          .or_default()
          .push(triangle);
      }
    }

    for (position, around) in &triangles_at {
      basis.smooth(*position, around, Direction::Tangent);
      basis.smooth(*position, around, Direction::Binormal);
    }

    basis.orthogonalize();

    let mut mesh: Mesh<u32> = basis.order_by_first_use()?;

    Self::shift_texture_coordinates(&mut mesh.vertices);

    Ok(mesh)
  }

  /// `GetGradients`: the directions `s` and `t` run in across a face, solved from its edges and their texture deltas.
  fn get_gradients([first, second, third]: [&MeshVertex; 3]) -> (Vector3d, Vector3d) {
    let along: Vector3d = &second.position - &first.position;
    let across: Vector3d = &third.position - &first.position;
    let (s1, t1): (f32, f32) = (
      second.texture_coordinate.x - first.texture_coordinate.x,
      second.texture_coordinate.y - first.texture_coordinate.y,
    );
    let (s2, t2): (f32, f32) = (
      third.texture_coordinate.x - first.texture_coordinate.x,
      third.texture_coordinate.y - first.texture_coordinate.y,
    );
    let determinant: f32 = s1 * t2 - s2 * t1;
    let scale: f32 = if determinant.abs() <= Self::DEGENERATE_MAPPING {
      if determinant > 0.0 { 1.0 } else { -1.0 }
    } else {
      1.0 / determinant
    };
    let tangent: Vector3d = Vector3d::new(
      t2 * along.x - t1 * across.x,
      t2 * along.y - t1 * across.y,
      t2 * along.z - t1 * across.z,
    );
    let binormal: Vector3d = Vector3d::new(
      s1 * across.x - s2 * along.x,
      s1 * across.y - s2 * along.y,
      s1 * across.z - s2 * along.z,
    );

    (&tangent * scale, &binormal * scale)
  }

  /// `ProcessTangents` and `ProcessBinormals`: groups the faces `around` a position by `direction`, then gives each
  /// group's vertices at the position its smoothed direction, splitting a vertex an earlier group already took.
  fn smooth(&mut self, position: PositionKey, around: &[usize], direction: Direction) {
    for triangle in around {
      self.triangles[*triangle].group = None;
      self.triangles[*triangle].is_handled = false;
    }

    let mut groups: Vec<Vec<usize>> = Vec::new();

    for triangle in around {
      if !self.triangles[*triangle].is_handled {
        self.build_groups(*triangle, around, &mut groups, direction);
      }
    }

    let mut taken: HashSet<usize> = HashSet::new();

    for group in &groups {
      let mut sum: Vector3d = Vector3d::ZERO;

      for triangle in group {
        sum += &self.get_direction(*triangle, direction);
      }

      let smoothed: Vector3d = Self::normalize(&sum);
      let mut used: Vec<usize> = Vec::new();

      for triangle in group {
        for corner in 0..3 {
          let index: usize = self.triangles[*triangle].indices[corner];

          if PositionKey::of(&self.vertices[index].position) != position {
            continue;
          }

          let index: usize = if taken.contains(&index) {
            let mut split: MeshVertex = self.vertices[index].clone();
            let split_index: usize = self.vertices.len();

            Self::set_direction(&mut split, direction, smoothed);
            self.vertices.push(split);
            self.renumber(index, split_index, group);

            split_index
          } else {
            Self::set_direction(&mut self.vertices[index], direction, smoothed);

            index
          };

          used.push(index);
        }
      }

      taken.extend(used);
    }
  }

  /// `BuildGroups`: puts `triangle` in the group of its first, then its second neighbour where its direction is close
  /// enough to theirs, or in a group of its own, then does the same for those neighbours.
  fn build_groups(&mut self, triangle: usize, around: &[usize], groups: &mut Vec<Vec<usize>>, direction: Direction) {
    let mut pending: Vec<usize> = vec![triangle];

    // The mender recurses into the first neighbour, then the second; a stack taken in reverse keeps that order.
    while let Some(triangle) = pending.pop() {
      if self.triangles[triangle].is_handled {
        continue;
      }

      let neighbours: [Option<usize>; 2] = self.find_neighbours(triangle, around);

      for neighbour in neighbours.into_iter().flatten() {
        if let Some(group) = self.triangles[neighbour].group
          && self.can_smooth(triangle, neighbour, direction)
        {
          groups[group].push(triangle);
          self.triangles[triangle].group = Some(group);
        }
      }

      if self.triangles[triangle].group.is_none() {
        self.triangles[triangle].group = Some(groups.len());
        groups.push(vec![triangle]);
      }

      self.triangles[triangle].is_handled = true;

      for neighbour in neighbours.into_iter().rev().flatten() {
        pending.push(neighbour);
      }
    }
  }

  /// `FindNeighbors`: the first two faces `around` the position that share an edge with `triangle`, by vertex.
  fn find_neighbours(&self, triangle: usize, around: &[usize]) -> [Option<usize>; 2] {
    let mut found = around
      .iter()
      .copied()
      .filter(|other| *other != triangle && self.shares_edge(triangle, *other));

    [found.next(), found.next()]
  }

  /// `SharesEdgeRespectSplits`: whether two faces have an edge between the same two vertices.
  fn shares_edge(&self, first: usize, second: usize) -> bool {
    let [a, b, c] = self.triangles[first].indices;
    let other: [usize; 3] = self.triangles[second].indices;
    let has_edge = |start: usize, end: usize| {
      (0..3).any(|edge| {
        let (from, to) = (other[edge], other[(edge + 1) % 3]);

        (from == start && to == end) || (from == end && to == start)
      })
    };

    has_edge(b, a) || has_edge(a, c) || has_edge(c, b)
  }

  /// Whether two faces' directions lie within [`Self::SMOOTHING_COSINE`], or both are no direction at all.
  fn can_smooth(&self, first: usize, second: usize, direction: Direction) -> bool {
    let (own, other) = (
      Self::normalize(&self.get_direction(first, direction)),
      Self::normalize(&self.get_direction(second, direction)),
    );

    own.dot(&other) >= Self::SMOOTHING_COSINE || (own == Vector3d::ZERO && other == Vector3d::ZERO)
  }

  /// `UpdateIndices`: points every face of `group` that used `from` at `to`.
  fn renumber(&mut self, from: usize, to: usize, group: &[usize]) {
    for triangle in group {
      for index in &mut self.triangles[*triangle].indices {
        if *index == from {
          *index = to;
        }
      }
    }
  }

  fn get_direction(&self, triangle: usize, direction: Direction) -> Vector3d {
    match direction {
      Direction::Tangent => self.triangles[triangle].tangent,
      Direction::Binormal => self.triangles[triangle].binormal,
    }
  }

  fn set_direction(vertex: &mut MeshVertex, direction: Direction, value: Vector3d) {
    match direction {
      Direction::Tangent => vertex.tangent = value,
      Direction::Binormal => vertex.binormal = value,
    }
  }

  /// `OrthogonalizeTangentsAndBinormals` as it runs: each smoothed tangent and binormal scaled by the inverse length of
  /// its Gram-Schmidt square to the normal, with a basis built from the normal where either collapses.
  fn orthogonalize(&mut self) {
    for vertex in &mut self.vertices {
      let normal: Vector3d = vertex.normal;
      let squared_tangent: Vector3d = &vertex.tangent - &(&normal * normal.dot(&vertex.tangent));
      let squared_binormal: Vector3d = &(&vertex.binormal - &(&normal * normal.dot(&vertex.binormal)))
        - &(&squared_tangent * squared_tangent.dot(&vertex.binormal));

      vertex.tangent = Self::scale_by_inverse_length(&vertex.tangent, &squared_tangent);
      vertex.binormal = Self::scale_by_inverse_length(&vertex.binormal, &squared_binormal);

      let (tangent_length, binormal_length): (f32, f32) = (vertex.tangent.length(), vertex.binormal.length());

      if tangent_length <= Self::COLLAPSED_LENGTH || binormal_length <= Self::COLLAPSED_LENGTH {
        if tangent_length > 0.5 {
          vertex.binormal = normal.cross(&vertex.tangent);
        } else if binormal_length > 0.5 {
          vertex.tangent = vertex.binormal.cross(&normal);
        } else {
          (vertex.tangent, vertex.binormal) = Self::get_basis_from_normal(&normal);
        }
      } else if vertex.binormal.dot(&vertex.tangent) > 0.999 {
        vertex.binormal = normal.cross(&vertex.tangent);
      }
    }
  }

  /// A tangent and binormal built from `normal` alone, for a vertex whose texture mapping gives it none: the axis
  /// further from the normal starts the basis.
  pub fn get_basis_from_normal(normal: &Vector3d) -> (Vector3d, Vector3d) {
    let start: Vector3d = if Vector3d::X.dot(normal) < Vector3d::Y.dot(normal) {
      Vector3d::X
    } else {
      Vector3d::Y
    };
    let tangent: Vector3d = normal.cross(&start);

    (tangent, normal.cross(&tangent))
  }

  /// `t_remove_isolated_verts`: the vertices the faces use, in the order the faces first reach them.
  fn order_by_first_use(self) -> XrfResult<Mesh<u32>> {
    let mut renumbered: Vec<Option<u32>> = vec![None; self.vertices.len()];
    let mut ordered: Vec<MeshVertex> = Vec::with_capacity(self.vertices.len());
    let mut faces: Vec<[u32; 3]> = Vec::with_capacity(self.triangles.len());

    for triangle in &self.triangles {
      let mut face: [u32; 3] = [0; 3];

      for (slot, index) in face.iter_mut().zip(triangle.indices) {
        *slot = match renumbered[index] {
          Some(number) => number,
          None => {
            let number: u32 = u32::try_from(ordered.len())
              .map_err(|_| XrfError::new_invalid_error("A split outgrows 32-bit indices once its tangents split it"))?;

            renumbered[index] = Some(number);
            ordered.push(self.vertices[index].clone());

            number
          }
        };
      }

      faces.push(face);
    }

    Ok(Mesh {
      vertices: ordered,
      faces,
    })
  }

  /// `OptimizeTextureCoordinates`: moves every texture coordinate by the whole tiles nearest the middle of the split's
  /// span, so they lie near the origin. The span's upper end starts from the smallest positive float, as the export's
  /// does, not from the lowest.
  pub fn shift_texture_coordinates(vertices: &mut [MeshVertex]) {
    let mut low: Vector2d = Vector2d::new(f32::MAX, f32::MAX);
    let mut high: Vector2d = Vector2d::new(f32::MIN_POSITIVE, f32::MIN_POSITIVE);

    for vertex in vertices.iter() {
      low = Vector2d::new(
        low.x.min(vertex.texture_coordinate.x),
        low.y.min(vertex.texture_coordinate.y),
      );
      high = Vector2d::new(
        high.x.max(vertex.texture_coordinate.x),
        high.y.max(vertex.texture_coordinate.y),
      );
    }

    let shift: Vector2d = Vector2d::new(
      ((high.x - low.x) / 2.0 + low.x).floor(),
      ((high.y - low.y) / 2.0 + low.y).floor(),
    );

    for vertex in vertices {
      vertex.texture_coordinate = Vector2d::new(
        vertex.texture_coordinate.x - shift.x,
        vertex.texture_coordinate.y - shift.y,
      );
    }
  }

  /// The mender's normalisation of a vector into itself, except that a vector of no length stays zero rather than
  /// turning into NaN.
  fn normalize(vector: &Vector3d) -> Vector3d {
    Self::scale_by_inverse_length(vector, vector)
  }

  /// `Vec3Normalize(out, measured)` as the mender has it: `out`'s own components times the inverse length of
  /// `measured`, zero where `measured` has no length.
  fn scale_by_inverse_length(out: &Vector3d, measured: &Vector3d) -> Vector3d {
    let length_squared: f32 = measured.length_squared();

    if length_squared > 0.0 {
      out * (1.0 / length_squared.sqrt())
    } else {
      Vector3d::ZERO
    }
  }
}
