use std::collections::HashMap;

use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::{EditableMeshNormals, EditableObjectFile, EditableSurface, EditableVMapKind};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_math::{EPS_L, Matrix4x4, Vector2d, Vector3d};

use crate::level::build::level_build_eigen::LevelBuildEigen;
use crate::level::build::level_build_vector::LevelBuildVector;
use crate::object::compile::geometry::source::mesh_corners::MeshCorners;

/// The faces of one kind of game material the static objects hold, split into the parts a build cuts out of the
/// geometry (`CGeomPartExtractor`, `LevelEditor/Edit/GeometryPartExtractor.cpp`): every face whose surface's game
/// material carries a flag, placed, its points welded within `EPS_L` (`VCPacked`), and the parts its faces make by
/// sharing a point. The ladders (`LevelBuildClimbables`) and the breakables (`LevelBuildBreakables`) are made of them.
pub(crate) struct LevelBuildParts {
  /// `SGameMtl`'s flag a surface's game material must carry for its faces to be gathered.
  flag: u32,
  pub(crate) faces: Vec<LevelBuildPartFace>,
  /// Each surface a face names, once per object and surface name.
  pub(crate) surfaces: Vec<EditableSurface>,
  surface_index: HashMap<(String, String), usize>,
  /// Each welded point, and the cells of `EPS_L` that find it.
  points: Vec<Vector3d>,
  cells: HashMap<[i64; 3], Vec<u32>>,
}

/// One face gathered: its corners placed, their welded points, their normals turned into the level, their texture
/// coordinates, and its surface by index.
pub(crate) struct LevelBuildPartFace {
  pub(crate) corners: [Vector3d; 3],
  pub(crate) points: [u32; 3],
  pub(crate) normals: [Vector3d; 3],
  pub(crate) texture_coordinates: [Vector2d; 3],
  pub(crate) surface: usize,
}

/// A part's box: its centre, and its axes and half sizes, each axis in a row.
pub(crate) struct LevelBuildPartBox {
  pub(crate) centre: [f64; 3],
  pub(crate) axes: [[f64; 3]; 3],
  pub(crate) half: [f64; 3],
}

impl LevelBuildParts {
  /// Parts of the faces whose game material carries `flag`.
  pub(crate) fn new(flag: u32) -> Self {
    Self {
      flag,
      faces: Vec::new(),
      surfaces: Vec::new(),
      surface_index: HashMap::new(),
      points: Vec::new(),
      cells: HashMap::new(),
    }
  }

  /// Adds every face of `object`, placed by `placed`, whose surface's game material carries the flag: each corner with
  /// the normal the editor gives it (`GenerateVNormals`) turned into the level, and the texture coordinate of its
  /// first map (`GetFaceTC`).
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, for a surface of a game material the library lacks, a face naming a surface
  /// or point its object lacks, or a mesh whose normals do not compute.
  pub(crate) fn add_object(
    &mut self,
    placed: &EditableSceneObject,
    object: &EditableObjectFile,
    game_materials: &GameMtlFile,
  ) -> XrfResult {
    let matrix: Matrix4x4 = placed.object.transform.to_matrix();
    let surfaces: &[EditableSurface] = object.surfaces.as_deref().unwrap_or_default();

    for mesh in object.meshes.iter().flatten() {
      let mut corner_data: Option<(EditableMeshNormals, Vec<[Vector2d; 3]>)> = None;

      for surface_faces in &mesh.surface_faces {
        let surface: &EditableSurface = surfaces
          .iter()
          .find(|surface| surface.name == surface_faces.surface)
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Object '{}' mesh '{}' names surface '{}' it does not hold",
              placed.reference, mesh.name, surface_faces.surface
            ))
          })?;
        let material: &GameMtlMaterial = game_materials
          .find_material_by_name(&surface.game_material)
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Object '{}', surface '{}' contain undefined game material.",
              placed.reference, surface.name
            ))
          })?;

        if material.flags & self.flag == 0 {
          continue;
        }

        if corner_data.is_none() {
          corner_data = Some((EditableMeshNormals::of(mesh)?, Self::read_texture_coordinates(mesh)?));
        }

        let (normals, uvs) = corner_data
          .as_ref()
          .map(|(normals, uvs)| (normals.corners.as_slice(), uvs.as_slice()))
          .unwrap_or_default();
        let surface: usize = self.intern_surface(&placed.reference, surface);

        for face in &surface_faces.faces {
          let index: usize = *face as usize;
          let corners = mesh
            .faces
            .get(index)
            .ok_or_else(|| XrfError::new_invalid_error(format!("Mesh '{}' names face {face}", mesh.name)))?
            .corners;
          let mut placed_corners: [Vector3d; 3] = [Vector3d::ZERO; 3];

          for (target, corner) in placed_corners.iter_mut().zip(&corners) {
            *target = matrix.transform_point(mesh.points.get(corner.point as usize).ok_or_else(|| {
              XrfError::new_invalid_error(format!("Mesh '{}' face {face} names a point it lacks", mesh.name))
            })?);
          }

          let normals: [Vector3d; 3] = normals[index].map(|normal| {
            let turned: Vector3d = matrix.transform_direction(&normal);

            &turned * (1.0 / turned.length().max(f32::EPSILON))
          });
          let points: [u32; 3] = placed_corners.map(|corner| self.weld(&corner));

          self.faces.push(LevelBuildPartFace {
            corners: placed_corners,
            points,
            normals,
            texture_coordinates: uvs[index],
            surface,
          });
        }
      }
    }

    Ok(())
  }

  /// The parts, each its faces in the order `recurse_tri` reaches them: from the first face no part holds yet, depth
  /// first through the faces around each of its points in the order they were added.
  pub(crate) fn get_parts(&self) -> Vec<Vec<usize>> {
    let around: Vec<Vec<usize>> = self.get_faces_around_points();
    let mut is_taken: Vec<bool> = vec![false; self.faces.len()];
    let mut parts: Vec<Vec<usize>> = Vec::new();

    for start in 0..self.faces.len() {
      if is_taken[start] {
        continue;
      }

      let mut part: Vec<usize> = Vec::new();
      // Each frame a face whose corners are walked, and how far.
      let mut stack: Vec<(usize, usize, usize)> = vec![(start, 0, 0)];

      is_taken[start] = true;
      part.push(start);

      while let Some((face, corner, next)) = stack.pop() {
        if corner == 3 {
          continue;
        }

        let faces: &[usize] = &around[self.faces[face].points[corner] as usize];

        if next == faces.len() {
          stack.push((face, corner + 1, 0));
          continue;
        }

        stack.push((face, corner, next + 1));

        let reached: usize = faces[next];

        if !is_taken[reached] {
          is_taken[reached] = true;
          part.push(reached);
          stack.push((reached, 0, 0));
        }
      }

      parts.push(part);
    }

    parts
  }

  /// Each face of `part`'s neighbours across an edge, as `SBPart::prepare` lists them: for each of its corners, the
  /// faces around that point in the order they were added, each sharing two points with it, once.
  pub(crate) fn get_edge_neighbours(&self, part: &[usize]) -> Vec<Vec<usize>> {
    let around: Vec<Vec<usize>> = self.get_faces_around_points();
    let mut neighbours: Vec<Vec<usize>> = vec![Vec::new(); self.faces.len()];

    for face in part {
      let points: [u32; 3] = self.faces[*face].points;

      for point in points {
        for other in &around[point as usize] {
          let shared: usize = self.faces[*other]
            .points
            .iter()
            .map(|it| points.iter().filter(|point| *point == it).count())
            .sum();

          if *other != *face && shared >= 2 {
            for (from, to) in [(*face, *other), (*other, *face)] {
              if !neighbours[from].contains(&to) {
                neighbours[from].push(to);
              }
            }
          }
        }
      }
    }

    part.iter().map(|face| std::mem::take(&mut neighbours[*face])).collect()
  }

  /// `RAPIDMinBox` (`ECore/WildMagic/RAPIDMinBox.cpp:230`) of `part`'s triangles: the area-weighted covariance of the
  /// triangles, its eigenvectors as axes, the one of the largest eigenvalue first, then the corners' extents along
  /// them; the axes made right-handed after (`ComputeOBB_RAPID`). Done in double precision about the first corner, since
  /// the editor's single precision on level coordinates turns a box by noise.
  pub(crate) fn fit_box(&self, part: &[usize]) -> Option<LevelBuildPartBox> {
    let corners: Vec<[f64; 3]> = part
      .iter()
      .flat_map(|face| self.faces[*face].corners)
      .map(|point| [f64::from(point.x), f64::from(point.y), f64::from(point.z)])
      .collect();
    let origin: [f64; 3] = *corners.first()?;
    let local: Vec<[f64; 3]> = corners
      .iter()
      .map(|corner| LevelBuildVector::sub(corner, &origin))
      .collect();
    let triangles: &[[[f64; 3]; 3]] = local.as_chunks::<3>().0;
    let areas: Vec<f64> = triangles
      .iter()
      .map(|triangle| {
        LevelBuildVector::length(&LevelBuildVector::cross(
          &LevelBuildVector::sub(&triangle[1], &triangle[0]),
          &LevelBuildVector::sub(&triangle[2], &triangle[0]),
        )) / 2.0
      })
      .collect();
    // A triangle of no area counts with the least area of the others, or 1 where every one has none.
    let least: f64 = areas
      .iter()
      .copied()
      .filter(|area| *area > 0.0)
      .reduce(f64::min)
      .unwrap_or(1.0);
    let (mut total, mut mean, mut second) = (0.0, [0.0; 3], [[0.0; 3]; 3]);

    for (triangle, area) in triangles.iter().zip(&areas) {
      let [p, q, r] = [triangle[0], triangle[1], triangle[2]];
      let centroid: [f64; 3] = [0, 1, 2].map(|axis| (p[axis] + q[axis] + r[axis]) / 3.0);
      let weight: f64 = if *area > 0.0 { *area } else { least };

      total += weight;

      for axis in 0..3 {
        mean[axis] += centroid[axis] * weight;
      }

      for (row, cells) in second.iter_mut().enumerate() {
        for (column, cell) in cells.iter_mut().enumerate() {
          let corners_moment: f64 = p[row] * p[column] + q[row] * q[column] + r[row] * r[column];

          *cell += if *area > 0.0 {
            area * (9.0 * centroid[row] * centroid[column] + corners_moment) / 12.0
          } else {
            corners_moment
          };
        }
      }
    }

    let covariance: [[f64; 3]; 3] =
      [0, 1, 2].map(|row| [0, 1, 2].map(|column| second[row][column] - mean[row] * mean[column] / total));
    let (values, vectors) = LevelBuildEigen::decompose(covariance);
    let largest: usize = (0..3).max_by(|a, b| values[*a].total_cmp(&values[*b]))?;
    let mut order: [usize; 3] = [0, 1, 2];

    order.swap(0, largest);

    let mut axes: [[f64; 3]; 3] = order.map(|column| LevelBuildEigen::get_column(&vectors, column));

    axes[0] = LevelBuildVector::cross(&axes[1], &axes[2]);
    axes[1] = LevelBuildVector::cross(&axes[2], &axes[0]);

    let (mut low, mut high) = ([f64::MAX; 3], [f64::MIN; 3]);

    for corner in &local {
      for (axis, (low, high)) in axes.iter().zip(low.iter_mut().zip(high.iter_mut())) {
        let along: f64 = LevelBuildVector::dot(axis, corner);

        *low = low.min(along);
        *high = high.max(along);
      }
    }

    let middle: [f64; 3] = [0, 1, 2].map(|axis| f64::midpoint(low[axis], high[axis]));
    let centre: [f64; 3] = [0, 1, 2]
      .map(|axis| origin[axis] + middle[0] * axes[0][axis] + middle[1] * axes[1][axis] + middle[2] * axes[2][axis]);

    Some(LevelBuildPartBox {
      centre,
      axes,
      half: [0, 1, 2].map(|axis| (high[axis] - low[axis]) / 2.0),
    })
  }

  /// Each welded point's faces, in the order they were added.
  fn get_faces_around_points(&self) -> Vec<Vec<usize>> {
    let mut around: Vec<Vec<usize>> = vec![Vec::new(); self.points.len()];

    for (index, face) in self.faces.iter().enumerate() {
      for point in face.points {
        around[point as usize].push(index);
      }
    }

    around
  }

  /// The surface's index, added the first time `object` names it.
  fn intern_surface(&mut self, object: &str, surface: &EditableSurface) -> usize {
    let key: (String, String) = (object.to_owned(), surface.name.clone());

    if let Some(index) = self.surface_index.get(&key) {
      return *index;
    }

    self.surfaces.push(surface.clone());
    self.surface_index.insert(key, self.surfaces.len() - 1);
    self.surfaces.len() - 1
  }

  /// Each face's texture coordinates of `mesh`, its first map's (`GetFaceTC`), zero where a corner names none.
  fn read_texture_coordinates(mesh: &xrf_editable_object::EditableMesh) -> XrfResult<Vec<[Vector2d; 3]>> {
    MeshCorners::without_normals(mesh).read_faces(|corner| {
      Ok(
        corner
          .entries
          .iter()
          .find(|entry| entry.vmap.kind == EditableVMapKind::Uv)
          .and_then(|entry| match entry.values {
            [u, v, ..] => Some(Vector2d::new(*u, *v)),
            _ => None,
          })
          .unwrap_or(Vector2d::ZERO),
      )
    })
  }

  /// The welded point at `point`: the first added within `EPS_L` of it on every axis, else a new one
  /// (`VCPacked::add_vert`).
  fn weld(&mut self, point: &Vector3d) -> u32 {
    let cell = |value: f32| (f64::from(value) / f64::from(EPS_L)).floor() as i64;
    let home: [i64; 3] = [cell(point.x), cell(point.y), cell(point.z)];

    for x in -1..=1 {
      for y in -1..=1 {
        for z in -1..=1 {
          let Some(candidates) = self.cells.get(&[home[0] + x, home[1] + y, home[2] + z]) else {
            continue;
          };

          if let Some(found) = candidates.iter().copied().find(|candidate| {
            let held: &Vector3d = &self.points[*candidate as usize];

            (held.x - point.x).abs() <= EPS_L && (held.y - point.y).abs() <= EPS_L && (held.z - point.z).abs() <= EPS_L
          }) {
            return found;
          }
        }
      }
    }

    let index: u32 = u32::try_from(self.points.len()).unwrap_or(u32::MAX);

    self.points.push(*point);
    self.cells.entry(home).or_default().push(index);

    index
  }
}
