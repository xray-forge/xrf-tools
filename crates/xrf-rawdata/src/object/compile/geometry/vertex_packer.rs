use std::collections::HashSet;

use xrf_editable_object::EditableMesh;
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Vector3d};

use crate::object::compile::geometry::export_vertex::ExportVertex;
use crate::object::compile::geometry::vertex_packing::VertexPacking;

/// Packs one split's corners into vertices as `CSkeletonCollectorPacked` and `CObjectOGFCollectorPacked` do, by the
/// grid and repeat rule of the [`VertexPacking`] given.
///
/// A corner reuses a vertex already packed when the two are similar: positions and normals within
/// [`Self::POSITION_DELTA`] per component, texture coordinates within [`Self::TEXTURE_DELTA`], and the same bones at
/// weights within [`Self::POSITION_DELTA`], a static vertex following none. Candidates come from a grid over the
/// object's box: a vertex is filed under its own cell and the cells one [`Self::POSITION_DELTA`] above it on each axis,
/// and a corner looks in its own cell only, so the grid decides which near vertices a corner can meet, and is kept as
/// the export lays it out. A corner matching none takes the position of the last vertex it stood within the delta of,
/// then is packed as a vertex of its own.
///
/// Indices are 32-bit here, where the export's are 16-bit: a split too big for one OGF child is cut into several once
/// its tangents are known (`SplitPartition`).
pub(crate) struct VertexPacker {
  pub(crate) vertices: Vec<ExportVertex>,
  pub(crate) faces: Vec<[u32; 3]>,
  /// Faces left out: degenerate, or repeating a face already added where the packing drops repeats.
  pub(crate) dropped_faces: usize,
  /// Each face's vertices sorted, which a face in any winding shares with every other winding of it; filled only
  /// where the packing drops repeats.
  face_sets: Option<HashSet<[u32; 3]>>,
  /// The vertices filed under each cell, in the order they were packed.
  cells: Vec<Vec<u32>>,
  cell_counts: [u32; 3],
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

  /// The object's box the grid spans, `CEditableObject::UpdateBox`: every mesh's stored bounds together, or `None`
  /// for an object without meshes.
  pub(crate) fn get_object_bounds(meshes: &[EditableMesh]) -> Option<Aabb> {
    meshes.iter().map(|mesh| mesh.bounds).reduce(|bounds, other| Aabb {
      min: bounds.min.min(&other.min),
      max: bounds.max.max(&other.max),
    })
  }

  /// A packer over `bounds`, the object's box, by `packing`'s grid and repeat rule.
  pub(crate) fn new(bounds: &Aabb, packing: &VertexPacking) -> Self {
    let grow: Vector3d = Vector3d::new(packing.growth, packing.growth, packing.growth);
    let epsilon: Vector3d = Vector3d::new(Self::EPSILON, Self::EPSILON, Self::EPSILON);
    let (low, high): (Vector3d, Vector3d) = (&bounds.min - &grow, &bounds.max + &grow);
    let scale: Vector3d = &(&high - &low) + &epsilon;
    let reach = |extent: f32, cells: u32| (extent / cells as f32 / 2.0).min(Self::POSITION_DELTA);
    let [cells_x, cells_y, cells_z] = packing.cells;

    Self {
      vertices: Vec::new(),
      faces: Vec::new(),
      dropped_faces: 0,
      face_sets: packing.drops_repeated_faces.then(HashSet::new),
      cells: vec![Vec::new(); packing.cells.iter().map(|cells| *cells as usize + 1).product()],
      cell_counts: packing.cells,
      minimum: &low - &epsilon,
      reach: Vector3d::new(
        reach(scale.x, cells_x),
        reach(scale.y, cells_y),
        reach(scale.z, cells_z),
      ),
      scale,
    }
  }

  /// Adds a face of `corners`, packing them, unless it is degenerate or repeats a face already added. Packing can move
  /// a corner onto a vertex it stood near, and `corners` keeps the moved positions, as the export's do.
  ///
  /// # Errors
  ///
  /// Returns an error when a corner stands outside the object's box, or the split outgrows 32-bit indices.
  pub(crate) fn add_face(&mut self, corners: &mut [ExportVertex; 3]) -> XrfResult {
    let is_near = |first: &Vector3d, second: &Vector3d| Self::is_within(first, second, Self::EPSILON);

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

  /// `VPack`: the vertex `corner` is, packed now when no similar one is.
  fn pack(&mut self, corner: &mut ExportVertex) -> XrfResult<u32> {
    let cell: [u32; 3] = self.get_cell(&corner.position)?;
    let mut near: Option<u32> = None;

    for candidate in &self.cells[self.get_cell_index(cell)] {
      let vertex: &ExportVertex = &self.vertices[*candidate as usize];

      if Self::is_within(&vertex.position, &corner.position, Self::POSITION_DELTA) {
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
      .map_err(|_| XrfError::new_invalid_error("A skeleton split holds more vertices than 32-bit indices reach"))?;

    self.vertices.push(corner.clone());
    self.file(index, cell)?;

    Ok(index)
  }

  /// Files a packed vertex under its cell and the cells one reach above it.
  fn file(&mut self, index: u32, cell: [u32; 3]) -> XrfResult {
    let position: Vector3d = self.vertices[index as usize].position;
    let above: [u32; 3] = self.get_cell(&(&position + &self.reach))?;
    let [x, y, z] = cell;
    let [x_above, y_above, z_above] = above;
    let mut filed: Vec<[u32; 3]> = vec![cell];

    for (is_x, is_y, is_z) in [
      (true, false, false),
      (false, true, false),
      (false, false, true),
      (true, true, false),
      (true, false, true),
      (false, true, true),
      (true, true, true),
    ] {
      let is_moved: bool = (!is_x || x_above != x) && (!is_y || y_above != y) && (!is_z || z_above != z);

      if is_moved {
        filed.push([
          if is_x { x_above } else { x },
          if is_y { y_above } else { y },
          if is_z { z_above } else { z },
        ]);
      }
    }

    for cell in filed {
      let cell_index: usize = self.get_cell_index(cell);

      self.cells[cell_index].push(index);
    }

    Ok(())
  }

  /// The cell `position` falls in, `iFloor((p - min) / scale * cells)` on each axis.
  fn get_cell(&self, position: &Vector3d) -> XrfResult<[u32; 3]> {
    let axis = |value: f32, minimum: f32, scale: f32, cells: u32| -> XrfResult<u32> {
      let cell: f32 = ((value - minimum) / scale * cells as f32).floor();

      if (0.0..=cells as f32).contains(&cell) {
        Ok(cell as u32)
      } else {
        Err(XrfError::new_invalid_error(format!(
          "A corner at {position} stands outside the object's box, which its meshes' bounds give"
        )))
      }
    };

    Ok([
      axis(position.x, self.minimum.x, self.scale.x, self.cell_counts[0])?,
      axis(position.y, self.minimum.y, self.scale.y, self.cell_counts[1])?,
      axis(position.z, self.minimum.z, self.scale.z, self.cell_counts[2])?,
    ])
  }

  fn get_cell_index(&self, [x, y, z]: [u32; 3]) -> usize {
    let [_, cells_y, cells_z] = self.cell_counts.map(|cells| cells as usize + 1);

    (x as usize * cells_y + y as usize) * cells_z + z as usize
  }

  /// `SSkelVert::similar`.
  fn is_similar(first: &ExportVertex, second: &ExportVertex) -> bool {
    first.bones.len() == second.bones.len()
      && first
        .bones
        .iter()
        .zip(&second.bones)
        .all(|(own, other)| own.bone == other.bone && (own.weight - other.weight).abs() < Self::POSITION_DELTA)
      && (first.texture_coordinate.x - second.texture_coordinate.x).abs() < Self::TEXTURE_DELTA
      && (first.texture_coordinate.y - second.texture_coordinate.y).abs() < Self::TEXTURE_DELTA
      && Self::is_within(&first.position, &second.position, Self::POSITION_DELTA)
      && Self::is_within(&first.normal, &second.normal, Self::POSITION_DELTA)
  }

  /// `Fvector::similar`: every component strictly within `delta`.
  fn is_within(first: &Vector3d, second: &Vector3d, delta: f32) -> bool {
    (first.x - second.x).abs() < delta && (first.y - second.y).abs() < delta && (first.z - second.z).abs() < delta
  }
}
