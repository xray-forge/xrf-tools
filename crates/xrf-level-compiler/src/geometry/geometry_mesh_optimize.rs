use xrf_error::{XrfError, XrfResult};
use xrf_geometry::FaceArea;
use xrf_math::{EPS_L, Vector2d, Vector3d};

use crate::geometry::geometry_face::GeometryFace;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_optimization::GeometryOptimization;

impl GeometryMesh {
  /// Cells of the weld's grid along each axis, past the first (`HDIM_X`, `HDIM_Y`, `HDIM_Z`).
  const WELD_CELLS: [usize; 3] = [56, 24, 56];

  /// The least perimeter a face's base coordinates may span, half a texel of a 4096 texture (`Face::Verify`).
  const TEXTURE_SPAN: f32 = 0.5 / 4096.0;

  /// How many refused faces an error lists by place.
  const LISTED_REFUSALS: usize = 7;

  /// `CBuild::PreOptimize` (`xrLC/xrPreOptimize.cpp:26`): welds the vertices, drops the faces the weld leaves
  /// degenerate, refuses the build over any face `Face::Verify` fails, and drops the vertices no face uses.
  ///
  /// A vertex welds to a kept one within `weld_distance` on every axis, strictly, found through a grid of
  /// [`Self::WELD_CELLS`] over the scene's box: a kept vertex is filed under its cell, and under the cells one reach
  /// above it on each axis (half a cell, at most `EPS_L`), and a vertex looks in its own cell only, in the order the
  /// vertices were kept. Vertices are taken in `build.prj`'s order. So across a cell's border a weld reaches no further
  /// than `EPS_L`, whatever the distance asked; and a distance of 0 welds nothing, not even vertices at one place.
  ///
  /// # Errors
  ///
  /// Returns an error listing the faces xrLC would refuse: an area under `EPS` or none at all, base coordinates
  /// spanning under [`Self::TEXTURE_SPAN`], or no normal to be had.
  pub fn optimize(&mut self, weld_distance: f32) -> XrfResult<GeometryOptimization> {
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
    let [cells_x, cells_y, cells_z] = Self::WELD_CELLS;
    let size: Vector3d = &self.bounds.max - &self.bounds.min;
    let minimum: Vector3d = self.bounds.min;
    let reach: Vector3d = Vector3d::new(
      (size.x / cells_x as f32 / 2.0).min(EPS_L),
      (size.y / cells_y as f32 / 2.0).min(EPS_L),
      (size.z / cells_z as f32 / 2.0).min(EPS_L),
    );
    let scale: Vector3d = Vector3d::new(
      cells_x as f32 / size.x,
      cells_y as f32 / size.y,
      cells_z as f32 / size.z,
    );
    let cell_of = |position: &Vector3d| -> [usize; 3] {
      let axis = |value: f32, minimum: f32, scale: f32, cells: usize| -> usize {
        (((value - minimum) * scale).floor().max(0.0) as usize).min(cells)
      };

      [
        axis(position.x, minimum.x, scale.x, cells_x),
        axis(position.y, minimum.y, scale.y, cells_y),
        axis(position.z, minimum.z, scale.z, cells_z),
      ]
    };
    let index_of = |[x, y, z]: [usize; 3]| (x * (cells_y + 1) + y) * (cells_z + 1) + z;
    let mut cells: Vec<Vec<u32>> = vec![Vec::new(); (cells_x + 1) * (cells_y + 1) * (cells_z + 1)];
    let mut welded: Vec<u32> = Vec::with_capacity(self.vertices.len());

    for (index, position) in self.vertices.iter().enumerate() {
      let cell: [usize; 3] = cell_of(position);
      let found: Option<u32> = cells[index_of(cell)]
        .iter()
        .copied()
        .find(|kept| Self::is_similar(&self.vertices[*kept as usize], position, weld_distance));

      if let Some(kept) = found {
        welded.push(kept);
        continue;
      }

      let index: u32 = index as u32;
      let above: [usize; 3] = cell_of(&(position + &reach));
      let [x, y, z] = cell;
      let [x_above, y_above, z_above] = above;

      welded.push(index);
      cells[index_of(cell)].push(index);

      // The cells one reach above, each where the reach crosses into it, in `PreOptimize`'s order.
      for (is_x, is_y, is_z) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
        (true, true, false),
        (true, false, true),
        (false, true, true),
        (true, true, true),
      ] {
        if (!is_x || x_above != x) && (!is_y || y_above != y) && (!is_z || z_above != z) {
          let filed: [usize; 3] = [
            if is_x { x_above } else { x },
            if is_y { y_above } else { y },
            if is_z { z_above } else { z },
          ];

          cells[index_of(filed)].push(index);
        }
      }
    }

    welded
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

  /// `Fvector::similar`: every component strictly within `delta`.
  fn is_similar(first: &Vector3d, second: &Vector3d, delta: f32) -> bool {
    (first.x - second.x).abs() < delta && (first.y - second.y).abs() < delta && (first.z - second.z).abs() < delta
  }

  /// `Fvector2::distance_to`.
  fn get_distance(first: &Vector2d, second: &Vector2d) -> f32 {
    ((first.x - second.x) * (first.x - second.x) + (first.y - second.y) * (first.y - second.y)).sqrt()
  }
}
