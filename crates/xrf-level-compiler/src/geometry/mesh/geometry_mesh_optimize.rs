use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{FaceArea, PackGrid};
use xrf_math::{Vector2d, Vector3d};

use crate::geometry::mesh::geometry_face::GeometryFace;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::mesh::geometry_optimization::GeometryOptimization;

impl GeometryMesh {
  /// Cells of the weld's grid along each axis, past the first (`HDIM_X`, `HDIM_Y`, `HDIM_Z`).
  const WELD_CELLS: [usize; 3] = [56, 24, 56];

  /// The least perimeter a face's base coordinates may span, half a texel of a 4096 texture (`Face::Verify`).
  const TEXTURE_SPAN: f32 = 0.5 / 4096.0;

  /// How many refused faces an error lists by place.
  const LISTED_REFUSALS: usize = 7;

  /// `CBuild::PreOptimize` (`xrLC/xrPreOptimize.cpp`): welds the vertices, drops the faces the weld leaves
  /// degenerate, refuses the build over any face `Face::Verify` fails, and drops the vertices no face uses.
  ///
  /// A vertex welds to a kept one within `weld_distance` on every axis, strictly, found through a [`PackGrid`] of
  /// `Self::WELD_CELLS` over the scene's box, the vertices taken in `build.prj`'s order. So across a cell's border a
  /// weld reaches no further than `EPS_L`, whatever the distance asked; and a distance of 0 welds nothing, not even
  /// vertices at one place.
  ///
  /// # Errors
  ///
  /// Returns an error listing the faces xrLC would refuse: an area under `EPS` or none at all, base coordinates
  /// spanning under `Self::TEXTURE_SPAN`, or no normal to be had.
  pub(crate) fn optimize(&mut self, weld_distance: f32) -> XrfResult<GeometryOptimization> {
    let welded: Vec<u32> = self.weld(weld_distance);
    let mut optimization: GeometryOptimization = GeometryOptimization {
      welded_vertices: welded
        .iter()
        .enumerate()
        .filter(|(index, kept)| *index != **kept as usize)
        .count(),
      ..GeometryOptimization::default()
    };
    let mut refused: Vec<String> = Vec::new();
    let mut refusals: usize = 0;
    let vertices: &[Vector3d] = &self.vertices;

    self.faces.retain_mut(|face| {
      face.vertices = face.vertices.map(|vertex| welded[vertex as usize]);

      let [a, b, c] = face.vertices;

      if a == b || a == c || b == c {
        optimization.degenerate_faces += 1;

        return false;
      }

      if let Some(reason) = Self::get_refusal(face, vertices) {
        refusals += 1;

        if refused.len() < Self::LISTED_REFUSALS {
          let [a, b, c] = face.get_corners(vertices);

          refused.push(format!("face {} {reason} at {a}, {b}, {c}", face.source));
        }
      }

      true
    });

    if refusals > 0 {
      return Err(XrfError::new_invalid_error(format!(
        "{refusals} faces xrLC refuses: {}",
        refused.join("; ")
      )));
    }

    optimization.isolated_vertices = self.drop_isolated_vertices();

    Ok(optimization)
  }

  /// The kept vertex each vertex welds to, itself where none.
  fn weld(&self, weld_distance: f32) -> Vec<u32> {
    let mut grid: PackGrid = PackGrid::new(Self::WELD_CELLS);
    let size: Vector3d = &self.bounds.max - &self.bounds.min;
    let reach: Vector3d = grid.get_reach(&size);
    let mut welded: Vec<u32> = Vec::with_capacity(self.vertices.len());

    for (index, position) in self.vertices.iter().enumerate() {
      let cell: [usize; 3] = self.get_weld_cell(position);
      let found: Option<u32> = grid
        .get_filed(cell)
        .iter()
        .copied()
        .find(|kept| self.vertices[*kept as usize].similar(position, weld_distance));

      if let Some(kept) = found {
        welded.push(kept);
        continue;
      }

      let index: u32 = index as u32;

      welded.push(index);
      grid.file(index, cell, self.get_weld_cell(&(position + &reach)));
    }

    welded
  }

  /// The weld grid's cell `position` falls in: `(p - min) * (cells / size)` floored, a point below the box in the first
  /// cell and one above in the last.
  fn get_weld_cell(&self, position: &Vector3d) -> [usize; 3] {
    let size: Vector3d = &self.bounds.max - &self.bounds.min;
    let minimum: &Vector3d = &self.bounds.min;
    let axis = |value: f32, minimum: f32, size: f32, cells: usize| -> usize {
      (((value - minimum) * (cells as f32 / size)).floor().max(0.0) as usize).min(cells)
    };
    let [cells_x, cells_y, cells_z] = Self::WELD_CELLS;

    [
      axis(position.x, minimum.x, size.x, cells_x),
      axis(position.y, minimum.y, size.y, cells_y),
      axis(position.z, minimum.z, size.z, cells_z),
    ]
  }

  /// Why `Face::Verify` refuses a face, if it does.
  fn get_refusal(face: &GeometryFace, vertices: &[Vector3d]) -> Option<&'static str> {
    let corners: [Vector3d; 3] = face.get_corners(vertices);

    if !FaceArea::is_enough(&corners) {
      return Some("of no area");
    }

    let [first, second, third] = face.texture_coordinates;
    let span: f32 =
      Self::get_distance(&first, &second) + Self::get_distance(&second, &third) + Self::get_distance(&third, &first);

    if span < Self::TEXTURE_SPAN {
      return Some("mapped to a point");
    }

    let [a, b, c] = corners.map(|corner| corner.to_array().map(f64::from));
    let along: [f64; 3] = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let across: [f64; 3] = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
    let cross: [f64; 3] = [
      along[1] * across[2] - along[2] * across[1],
      along[2] * across[0] - along[0] * across[2],
      along[0] * across[1] - along[1] * across[0],
    ];

    // `CalcNormal2`'s last resort, `Failure` where even double precision finds no normal.
    ((cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() < f64::MIN_POSITIVE)
      .then_some("with no normal")
  }

  /// Drops the vertices no face uses, keeping the others in order, and renumbers the faces.
  fn drop_isolated_vertices(&mut self) -> usize {
    let mut is_used: Vec<bool> = vec![false; self.vertices.len()];

    for face in &self.faces {
      for vertex in face.vertices {
        is_used[vertex as usize] = true;
      }
    }

    let mut renumbered: Vec<u32> = vec![u32::MAX; self.vertices.len()];
    let mut kept: Vec<Vector3d> = Vec::with_capacity(self.vertices.len());

    for (index, position) in self.vertices.iter().enumerate() {
      if is_used[index] {
        renumbered[index] = kept.len() as u32;
        kept.push(*position);
      }
    }

    let isolated: usize = self.vertices.len() - kept.len();

    for face in &mut self.faces {
      face.vertices = face.vertices.map(|vertex| renumbered[vertex as usize]);
    }

    self.vertices = kept;

    isolated
  }

  /// `Fvector2::distance_to`.
  fn get_distance(first: &Vector2d, second: &Vector2d) -> f32 {
    ((first.x - second.x) * (first.x - second.x) + (first.y - second.y) * (first.y - second.y)).sqrt()
  }
}
