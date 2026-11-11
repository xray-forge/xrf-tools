use std::collections::HashMap;

use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::{EditableMeshNormals, EditableObjectFile, EditableSurface};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_math::{Affine3d, EPS_L, Matrix4x4, Vector3d};
use xrf_spawn::{
  AlifeObject, AlifeObjectAbstract, AlifeObjectClimable, AlifeObjectInherited, AlifeObjectShape, ClsId, Shape,
  SpawnPacketHeader,
};

use crate::level::build::level_build_eigen::LevelBuildEigen;
use crate::level::build::level_build_vector::LevelBuildVector;

/// The ladders a build cuts out of the geometry (`ESceneObjectTool::ExportClimableObjects`,
/// `ESceneObjectToolsExportBreakable.cpp:236`): every face of a climbable game material of the static objects, placed,
/// split into parts that share a point (`CGeomPartExtractor`, points welded within `EPS_L`), each part a
/// `climable_object` named `clmbl#<n>` standing at its box's centre (`ComputeOBB_RAPID`), turned so its thinnest axis
/// faces the way its faces do (`OrientToNorm`), its shape the box, its game material its last face's.
#[derive(Default)]
pub(crate) struct LevelBuildClimbables {
  faces: Vec<ClimbableFace>,
  /// Each welded point, and the cells of `EPS_L` that find it.
  points: Vec<Vector3d>,
  cells: HashMap<[i64; 3], Vec<u32>>,
}

/// One climbable face: its corners placed, their welded points, their normals, and its game material.
struct ClimbableFace {
  corners: [Vector3d; 3],
  points: [u32; 3],
  normals: [Vector3d; 3],
  game_material: String,
}

/// A part's box: its centre, and its axes and half sizes, each axis in a row.
struct PartBox {
  centre: [f64; 3],
  axes: [[f64; 3]; 3],
  half: [f64; 3],
}

/// The ladders a build made of the climbable faces, and the parts it could not make one of.
pub(crate) struct LevelBuildClimbableExport {
  pub(crate) entities: Vec<AlifeObject>,
  /// Each part refused, by its name, with why.
  pub(crate) refused: Vec<(String, String)>,
}

impl LevelBuildClimbables {
  /// The class every ladder is of, and what names it.
  const SECTION: &'static str = "climable_object";
  const NAME_PREFIX: &'static str = "clmbl#";

  /// Adds every face of a climbable game material of `object`, placed by `placed`, each corner with the normal the
  /// editor gives it (`GenerateVNormals`) turned into the level.
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
      let mut normals: Option<EditableMeshNormals> = None;

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

        if material.flags & GameMtlMaterial::FLAG_CLIMABLE == 0 {
          continue;
        }

        if normals.is_none() {
          normals = Some(EditableMeshNormals::of(mesh)?);
        }

        let corner_normals: &[[Vector3d; 3]] = normals.as_ref().map(|it| it.corners.as_slice()).unwrap_or_default();

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

          let normals: [Vector3d; 3] = corner_normals[index].map(|normal| {
            let turned: Vector3d = matrix.transform_direction(&normal);

            &turned * (1.0 / turned.length().max(f32::EPSILON))
          });
          let points: [u32; 3] = placed_corners.map(|corner| self.weld(&corner));

          self.faces.push(ClimbableFace {
            corners: placed_corners,
            points,
            normals,
            game_material: surface.game_material.clone(),
          });
        }
      }
    }

    Ok(())
  }

  /// A ladder of each part, numbered by part; a part some face of which shares no edge with another, or whose
  /// thinnest axis is not the one its faces face along, refused as the Build refuses it.
  pub(crate) fn into_export(self) -> LevelBuildClimbableExport {
    let mut export: LevelBuildClimbableExport = LevelBuildClimbableExport {
      entities: Vec::new(),
      refused: Vec::new(),
    };

    for (index, part) in self.get_parts().iter().enumerate() {
      let name: String = format!("{}{index}", Self::NAME_PREFIX);

      match self.get_entity(&name, part) {
        Ok(entity) => export.entities.push(entity),
        Err(reason) => export.refused.push((name, reason)),
      }
    }

    export
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

  /// The parts, each its faces in the order `recurse_tri` reaches them: from the first face no part holds yet, depth
  /// first through the faces around each of its points in the order they were added.
  fn get_parts(&self) -> Vec<Vec<usize>> {
    let mut around: Vec<Vec<usize>> = vec![Vec::new(); self.points.len()];

    for (index, face) in self.faces.iter().enumerate() {
      for point in face.points {
        around[point as usize].push(index);
      }
    }

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

  /// The ladder `part` makes, or why it makes none.
  fn get_entity(&self, name: &str, part: &[usize]) -> Result<AlifeObject, String> {
    if part.iter().any(|face| !self.has_edge_neighbour(*face, part)) {
      return Err(String::from(
        "a face of it shares no edge with another (\"Error face found\")",
      ));
    }

    let corners: Vec<[f64; 3]> = part
      .iter()
      .flat_map(|face| self.faces[*face].corners)
      .map(|point| [f64::from(point.x), f64::from(point.y), f64::from(point.z)])
      .collect();
    let mut fitted: PartBox = Self::fit_box(&corners).ok_or_else(|| String::from("its corners hold no box"))?;
    let normal: [f64; 3] = part
      .iter()
      .flat_map(|face| self.faces[*face].normals)
      .fold([0.0; 3], |sum, normal| {
        [
          sum[0] + f64::from(normal.x),
          sum[1] + f64::from(normal.y),
          sum[2] + f64::from(normal.z),
        ]
      });
    let local: [f64; 3] = fitted.axes.map(|axis| LevelBuildVector::dot(&axis, &normal));

    if !Self::orient_to_normal(&local, &mut fitted) {
      return Err(String::from(
        "its thinnest axis is not the one its faces face along most (\"Invalid climable object found\")",
      ));
    }

    let to_vector = |value: [f64; 3]| Vector3d::new(value[0] as f32, value[1] as f32, value[2] as f32);
    let frame: Affine3d = Affine3d {
      x_axis: to_vector(fitted.axes[0]),
      y_axis: to_vector(fitted.axes[1]),
      z_axis: to_vector(fitted.axes[2]),
      translation: to_vector(fitted.centre),
    };
    let half: Vector3d = to_vector(fitted.half);

    Ok(AlifeObject {
      header: SpawnPacketHeader {
        id: u16::MAX,
        net_action: 1,
        section: String::from(Self::SECTION),
        name: name.to_owned(),
        script_game_id: 0,
        script_rp: 0xFE,
        position: frame.translation,
        direction: frame.get_xyz(),
        respawn_time: 0,
        parent_id: u16::MAX,
        phantom_id: u16::MAX,
        script_flags: 33,
        version: 128,
        game_type: u16::MAX,
        script_version: 12,
        client_data: Vec::new(),
        spawn_id: u16::MAX,
      },
      clsid: ClsId::OClmbl,
      inherited: AlifeObjectInherited::CseAlifeObjectClimable(Box::new(AlifeObjectClimable {
        base: AlifeObjectShape {
          base: AlifeObjectAbstract {
            game_vertex_id: u16::MAX,
            distance: 0.0,
            direct_control: 1,
            level_vertex_id: u32::MAX,
            flags: 0xFFFF_FFBF,
            custom_data: String::new(),
            story_id: u32::MAX,
            spawn_story_id: u32::MAX,
          },
          shape: vec![Shape::Box(Affine3d {
            x_axis: Vector3d::new(half.x, 0.0, 0.0),
            y_axis: Vector3d::new(0.0, half.y, 0.0),
            z_axis: Vector3d::new(0.0, 0.0, half.z),
            translation: Vector3d::ZERO,
          })],
        },
        game_material: part
          .last()
          .map(|face| self.faces[*face].game_material.clone())
          .unwrap_or_default(),
      })),
    })
  }

  /// Whether face `face` shares two points with another face of `part` (`SBPart::prepare`'s adjacency).
  fn has_edge_neighbour(&self, face: usize, part: &[usize]) -> bool {
    let points: [u32; 3] = self.faces[face].points;

    part.iter().any(|other| {
      *other != face
        && self.faces[*other]
          .points
          .iter()
          .map(|point| points.iter().filter(|it| *it == point).count())
          .sum::<usize>()
          >= 2
    })
  }

  /// `RAPIDMinBox` (`ECore/WildMagic/RAPIDMinBox.cpp:230`) of triangles `corners`: the area-weighted covariance of the
  /// triangles, its eigenvectors as axes, the one of the largest eigenvalue first, then the corners' extents along
  /// them; the axes made right-handed after (`ComputeOBB_RAPID`). Done in double precision about the first corner, since
  /// the editor's single precision on level coordinates turns a box by noise.
  fn fit_box(corners: &[[f64; 3]]) -> Option<PartBox> {
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

    Some(PartBox {
      centre,
      axes,
      half: [0, 1, 2].map(|axis| (high[axis] - low[axis]) / 2.0),
    })
  }

  /// `OrientToNorm` (`ESceneObjectToolsExportBreakable.cpp:214`): false unless the axis the faces' summed normal
  /// `local` (in the box's frame) leans on most is the box's thinnest; where it leans the negative way, that axis and
  /// the next are turned round, so the box stays right-handed.
  fn orient_to_normal(local: &[f64; 3], fitted: &mut PartBox) -> bool {
    let (mut most, mut thinnest) = (0, 0);

    for axis in 1..3 {
      if local[axis].abs() > local[most].abs() {
        most = axis;
      }

      if fitted.half[axis] < fitted.half[thinnest] {
        thinnest = axis;
      }
    }

    if most != thinnest {
      return false;
    }

    if local[most] < 0.0 {
      for axis in [most, (most + 1) % 3] {
        fitted.axes[axis] = fitted.axes[axis].map(|value| -value);
      }
    }

    true
  }
}
