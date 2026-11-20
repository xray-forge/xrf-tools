use std::collections::HashSet;

use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Vector3d};

use crate::mesh::mesh::Mesh;
use crate::mesh::mesh_vertex::MeshVertex;
use crate::mesh::vertex_packing::VertexPacking;
use crate::weld::pack_grid::PackGrid;

/// Packs one split's corners into vertices as `CSkeletonCollectorPacked` and `CObjectOGFCollectorPacked` do, by the
/// grid and repeat rule of the [`VertexPacking`] given.
///
/// Indices are 32-bit, where the export's are 16-bit: `SplitPartition` cuts a split too big for one OGF child once its
/// tangents are known.
pub struct VertexPacker {
  vertices: Vec<MeshVertex>,
  faces: Vec<[u32; 3]>,
  /// Faces left out: degenerate, or repeating a face already added where the packing drops repeats.
  dropped_faces: usize,
  /// Each face's vertices sorted, which a face in any winding shares with every other winding of it; filled only
  /// where the packing drops repeats.
  face_sets: Option<HashSet<[u32; 3]>>,
  /// The packed vertices by cell.
  grid: PackGrid,
  minimum: Vector3d,
  scale: Vector3d,
  /// How far above itself on each axis a vertex is filed.
  reach: Vector3d,
}

impl VertexPacker {
  /// `g_EpsSkelPositionDelta`, `EPS_L`: how close positions, normals and weights pack together.
  const POSITION_DELTA: f32 = 1e-3;
  /// `EPS_S`: how close texture coordinates pack together.
  const TEXTURE_DELTA: f32 = 1e-7;
  /// `EPS`: how close two corners of one face stand before the face is degenerate, and how far the grid reaches past
  /// the object's box and its growth.
  const EPSILON: f32 = 1e-5;

  /// A packer over `bounds`, the object's box, by `packing`'s grid and repeat rule.
  pub fn new(bounds: &Aabb, packing: &VertexPacking) -> Self {
    let grow: Vector3d = Vector3d::new(packing.growth, packing.growth, packing.growth);
    let epsilon: Vector3d = Vector3d::new(Self::EPSILON, Self::EPSILON, Self::EPSILON);
    let (low, high): (Vector3d, Vector3d) = (&bounds.min - &grow, &bounds.max + &grow);
    let scale: Vector3d = &(&high - &low) + &epsilon;
    let grid: PackGrid = PackGrid::new(packing.cells.map(|cells| cells as usize));

    Self {
      vertices: Vec::new(),
      faces: Vec::new(),
      dropped_faces: 0,
      face_sets: packing.drops_repeated_faces.then(HashSet::new),
      reach: grid.get_reach(&scale),
      grid,
      minimum: &low - &epsilon,
      scale,
    }
  }

  /// The vertices packed so far, by index.
  pub fn get_vertices(&self) -> &[MeshVertex] {
    &self.vertices
  }

  /// The faces added so far, over [`Self::get_vertices`].
  pub fn get_faces(&self) -> &[[u32; 3]] {
    &self.faces
  }

  /// How many faces were left out: degenerate, or repeating a face already added where the packing drops repeats.
  pub fn get_dropped_faces(&self) -> usize {
    self.dropped_faces
  }

  /// The split packed: its vertices and faces.
  pub fn into_mesh(self) -> Mesh<u32> {
    Mesh {
      vertices: self.vertices,
      faces: self.faces,
    }
  }

  /// Adds a face of `corners`, packing them, unless it is degenerate or repeats a face already added. Packing can move
  /// a corner onto a vertex it stood near, and `corners` keeps the moved positions, as the export's do.
  ///
  /// # Errors
  ///
  /// Returns an error when a corner stands outside the box the packer spans, or the split outgrows 32-bit indices.
  pub fn add_face(&mut self, corners: &mut [MeshVertex; 3]) -> XrfResult {
    let is_near = |first: &Vector3d, second: &Vector3d| first.similar(second, Self::EPSILON);

    if is_near(&corners[0].position, &corners[1].position)
      || is_near(&corners[0].position, &corners[2].position)
      || is_near(&corners[1].position, &corners[2].position)
    {
      self.dropped_faces += 1;

      return Ok(());
    }

    let face: [u32; 3] = [
      self.pack(&mut corners[0])?,
      self.pack(&mut corners[1])?,
      self.pack(&mut corners[2])?,
    ];
    let mut sorted: [u32; 3] = face;

    sorted.sort_unstable();

    let is_repeat: bool = self
      .face_sets
      .as_mut()
      .is_some_and(|face_sets| !face_sets.insert(sorted));

    if sorted[0] == sorted[1] || sorted[1] == sorted[2] || is_repeat {
      self.dropped_faces += 1;
    } else {
      self.faces.push(face);
    }

    Ok(())
  }

  /// `VPack`: the vertex `corner` is, packed now when no similar one is, at the position of the last vertex it stood
  /// near.
  fn pack(&mut self, corner: &mut MeshVertex) -> XrfResult<u32> {
    let cell: [usize; 3] = self.get_cell(&corner.position)?;
    let mut near: Option<u32> = None;

    for candidate in self.grid.get_filed(cell) {
      let vertex: &MeshVertex = &self.vertices[*candidate as usize];

      if vertex.position.similar(&corner.position, Self::POSITION_DELTA) {
        if Self::is_similar(vertex, corner) {
          return Ok(*candidate);
        }

        near = Some(*candidate);
      }
    }

    if let Some(near) = near {
      corner.position = self.vertices[near as usize].position;
    }

    let index: u32 = u32::try_from(self.vertices.len())
      .map_err(|_| XrfError::new_invalid_error("A split holds more vertices than 32-bit indices reach"))?;

    let above: [usize; 3] = self.get_cell(&(&corner.position + &self.reach))?;

    self.vertices.push(corner.clone());
    self.grid.file(index, cell, above);

    Ok(index)
  }

  /// The cell `position` falls in, `iFloor((p - min) / scale * cells)` on each axis.
  fn get_cell(&self, position: &Vector3d) -> XrfResult<[usize; 3]> {
    let axis = |value: f32, minimum: f32, scale: f32, cells: usize| -> XrfResult<usize> {
      let cell: f32 = ((value - minimum) / scale * cells as f32).floor();

      if (0.0..=cells as f32).contains(&cell) {
        Ok(cell as usize)
      } else {
        Err(XrfError::new_invalid_error(format!(
          "A corner at {position} stands outside the box the packing spans, an object's its meshes' stored bounds"
        )))
      }
    };
    let [cells_x, cells_y, cells_z] = self.grid.get_counts();

    Ok([
      axis(position.x, self.minimum.x, self.scale.x, cells_x)?,
      axis(position.y, self.minimum.y, self.scale.y, cells_y)?,
      axis(position.z, self.minimum.z, self.scale.z, cells_z)?,
    ])
  }

  /// `SSkelVert::similar`.
  fn is_similar(first: &MeshVertex, second: &MeshVertex) -> bool {
    first.bones.len() == second.bones.len()
      && first
        .bones
        .iter()
        .zip(&second.bones)
        .all(|(own, other)| own.bone == other.bone && (own.weight - other.weight).abs() < Self::POSITION_DELTA)
      && (first.texture_coordinate.x - second.texture_coordinate.x).abs() < Self::TEXTURE_DELTA
      && (first.texture_coordinate.y - second.texture_coordinate.y).abs() < Self::TEXTURE_DELTA
      && first.position.similar(&second.position, Self::POSITION_DELTA)
      && first.normal.similar(&second.normal, Self::POSITION_DELTA)
  }
}
