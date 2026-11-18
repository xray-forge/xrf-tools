use std::collections::HashMap;

use glam::{DVec3, Vec3};
use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::{EditableMeshNormals, EditableObjectFile, EditableSurface, EditableVMapKind};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_geometry::{PointWelder, SymmetricEigen};
use xrf_math::{Matrix4x4, Vector2d, Vector3d};

use crate::level::build::level_build_part_box::LevelBuildPartBox;
use crate::level::build::level_build_part_face::LevelBuildPartFace;
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
  /// The faces' corners welded into points.
  points: PointWelder,
}

impl LevelBuildParts {
  /// Parts of the faces whose game material carries `flag`.
  pub(crate) fn new(flag: u32) -> Self {
    Self {
      flag,
      faces: Vec::new(),
      surfaces: Vec::new(),
      surface_index: HashMap::new(),
      points: PointWelder::default(),
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
          let points: [u32; 3] = placed_corners.map(|corner| self.points.weld(&corner));

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
    let corners: Vec<DVec3> = part
      .iter()
      .flat_map(|face| self.faces[*face].corners)
      .map(|point| Vec3::from(point).as_dvec3())
      .collect();
    let origin: DVec3 = *corners.first()?;
    let local: Vec<DVec3> = corners.iter().map(|corner| *corner - origin).collect();
    let triangles: &[[DVec3; 3]] = local.as_chunks::<3>().0;
    let areas: Vec<f64> = triangles
      .iter()
      .map(|triangle| (triangle[1] - triangle[0]).cross(triangle[2] - triangle[0]).length() / 2.0)
      .collect();
    // A triangle of no area counts with the least area of the others, or 1 where every one has none.
    let least: f64 = areas
      .iter()
      .copied()
      .filter(|area| *area > 0.0)
      .reduce(f64::min)
      .unwrap_or(1.0);
    let (mut total, mut mean, mut second) = (0.0, DVec3::ZERO, [[0.0; 3]; 3]);

    for (triangle, area) in triangles.iter().zip(&areas) {
      let [p, q, r] = *triangle;
      let centroid: DVec3 = (p + q + r) / 3.0;
      let weight: f64 = if *area > 0.0 { *area } else { least };

      total += weight;
      mean += centroid * weight;

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
    let eigen: SymmetricEigen = SymmetricEigen::of(covariance);
    let mut order: [usize; 3] = [0, 1, 2];

    order.swap(0, eigen.get_greatest());

    let mut axes: [DVec3; 3] = order.map(|column| eigen.vectors[column]);

    axes[0] = axes[1].cross(axes[2]);
    axes[1] = axes[2].cross(axes[0]);

    let (mut low, mut high) = (DVec3::MAX, DVec3::MIN);

    for corner in &local {
      let along: DVec3 = DVec3::from_array(axes.map(|axis| axis.dot(*corner)));

      low = low.min(along);
      high = high.max(along);
    }

    let middle: DVec3 = DVec3::from_array([0, 1, 2].map(|axis| f64::midpoint(low[axis], high[axis])));

    Some(LevelBuildPartBox {
      centre: origin + middle.x * axes[0] + middle.y * axes[1] + middle.z * axes[2],
      axes,
      half: (high - low) / 2.0,
    })
  }

  /// Each welded point's faces, in the order they were added.
  fn get_faces_around_points(&self) -> Vec<Vec<usize>> {
    let mut around: Vec<Vec<usize>> = vec![Vec::new(); self.points.get_points().len()];

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
}
