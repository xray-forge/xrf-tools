use xrf_editable_level::{EditableSceneObject, EditableWallmarkTool};
use xrf_editable_object::{EditableMesh, EditableObjectFile, EditableSurface, EditableVMapKind};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::BuildPrjFace;
use xrf_math::{EPS, Matrix4x4, Vector2d, Vector3d};

use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::LevelBuildTables;
use crate::object::compile::geometry::source::mesh_corners::MeshCorners;

/// The static geometry a build hands xrLC: every vertex in world space, the faces over them and a smoothing group per
/// face, as `SceneBuilder::BuildMesh` (`BuilderRemote.cpp:587`) gathers them from the static objects and
/// `ESceneWallmarkTool::ExportStatic` (`ESceneWallmarkTools.cpp:954`) from the wall marks.
#[derive(Default)]
pub(crate) struct LevelBuildGeometry {
  pub(crate) vertices: Vec<Vector3d>,
  pub(crate) faces: Vec<BuildPrjFace>,
  pub(crate) smoothing_groups: Vec<u32>,
  /// Faces of surfaces the build leaves out: occluders, breakables and ladders, which other exports take.
  pub(crate) skipped_faces: usize,
  /// Faces of no area, which the build drops (`"has %d invalid face(s). Removed."`).
  pub(crate) invalid_faces: usize,
}

impl LevelBuildGeometry {
  /// The game material whose faces go to `level.hom` rather than the geometry.
  const OCCLUDER_MATERIAL: &'static str = "materials\\occ";

  /// The compiler shader of a wall mark's faces, `COMPILER_SHADER` (`ESceneWallmarkTools.cpp`).
  const WALLMARK_COMPILER_SHADER: &'static str = "def_shaders\\def_vertex_ghost_no_shadow";

  /// `D3DFVF_TEXCOUNT_MASK` and its shift, which say how many texture coordinate sets a surface's corners carry.
  const TEXTURE_COUNT_MASK: u32 = 0xF00;
  const TEXTURE_COUNT_SHIFT: u32 = 8;

  /// Adds the faces of each mesh of `object`, placed by `placed`, in the sector naming the mesh.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, for a surface of a game material the library lacks or of a dynamic one, of
  /// other than one texture coordinate set, or naming no texture; or a mesh whose faces name points or maps it lacks.
  pub(crate) fn add_object(
    &mut self,
    placed: &EditableSceneObject,
    object: &EditableObjectFile,
    sectors: &LevelBuildSectors,
    tables: &mut LevelBuildTables,
    game_materials: &GameMtlFile,
  ) -> XrfResult {
    let matrix: Matrix4x4 = placed.object.transform.to_matrix();
    let surfaces: &[EditableSurface] = object.surfaces.as_deref().unwrap_or_default();

    for mesh in object.meshes.iter().flatten() {
      let sector: u16 = sectors.find(&placed.object.name, &mesh.name);
      let offset: u32 = u32::try_from(self.vertices.len())
        .map_err(|_| XrfError::new_invalid_error("A build holds more vertices than its u32 indices number"))?;
      let uvs: Vec<[Vector2d; 3]> = MeshCorners::without_normals(mesh).read_faces(|corner| {
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
      })?;

      self
        .vertices
        .extend(mesh.points.iter().map(|point| matrix.transform_point(point)));

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

        if surface.game_material == Self::OCCLUDER_MATERIAL {
          self.skipped_faces += surface_faces.faces.len();
          continue;
        }

        let game_material: &GameMtlMaterial = game_materials
          .find_material_by_name(&surface.game_material)
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Surface '{}' of '{}' contains undefined game material '{}'",
              surface.name, placed.reference, surface.game_material
            ))
          })?;

        if game_material.flags & (GameMtlMaterial::FLAG_BREAKABLE | GameMtlMaterial::FLAG_CLIMABLE) != 0 {
          self.skipped_faces += surface_faces.faces.len();
          continue;
        }

        if game_material.flags & GameMtlMaterial::FLAG_DYNAMIC != 0 {
          return Err(XrfError::new_invalid_error(format!(
            "Surface '{}' of '{}' contains non-static game material '{}'",
            surface.name, placed.reference, surface.game_material
          )));
        }

        if (surface.fvf & Self::TEXTURE_COUNT_MASK) >> Self::TEXTURE_COUNT_SHIFT != 1 {
          return Err(XrfError::new_invalid_error(format!(
            "Surface '{}' of '{}' contains more than 1 texture refs",
            surface.name, placed.reference
          )));
        }

        let material: u16 = tables.get_material(
          &surface.shader,
          Some(&surface.compiler_shader),
          &surface.texture,
          sector,
        )?;

        for face in &surface_faces.faces {
          self.add_face(
            mesh,
            *face as usize,
            offset,
            &uvs,
            material,
            game_material.id,
            surface.is_two_sided(),
          )?;
        }
      }
    }

    Ok(())
  }

  /// Adds one face of `mesh`, and its back where its surface is two-sided; a face of no area is counted and left out.
  #[expect(
    clippy::too_many_arguments,
    reason = "the face's mesh, its place in the build and what dresses it"
  )]
  fn add_face(
    &mut self,
    mesh: &EditableMesh,
    face: usize,
    offset: u32,
    uvs: &[[Vector2d; 3]],
    material: u16,
    game_material: u32,
    is_two_sided: bool,
  ) -> XrfResult {
    let corners = &mesh
      .faces
      .get(face)
      .ok_or_else(|| XrfError::new_invalid_error(format!("Mesh '{}' names face {face}", mesh.name)))?
      .corners;
    let points: [u32; 3] = corners.each_ref().map(|corner| corner.point);
    let area: f32 = Self::get_area(&points.map(|point| mesh.points[point as usize]));

    if !area.is_finite() || area < EPS {
      self.invalid_faces += 1;
      return Ok(());
    }

    let smoothing: u32 = mesh
      .smoothing_groups
      .as_ref()
      .and_then(|groups| groups.get(face).copied())
      .unwrap_or(EditableMesh::FLAT_FACE_SMOOTHING);
    let uv: [Vector2d; 3] = uvs[face];

    self.faces.push(BuildPrjFace {
      vertices: points.map(|point| point + offset),
      texture_coordinates: uv,
      material,
      material_padding: 0,
      game_material,
    });
    self
      .smoothing_groups
      .push(smoothing & !EditableMesh::BACK_FACE_SMOOTHING_FLAG);

    if is_two_sided {
      self.faces.push(BuildPrjFace {
        vertices: [points[2], points[1], points[0]].map(|point| point + offset),
        texture_coordinates: [uv[2], uv[1], uv[0]],
        material,
        material_padding: 0,
        game_material,
      });
      self
        .smoothing_groups
        .push(smoothing | EditableMesh::BACK_FACE_SMOOTHING_FLAG);
    }

    Ok(())
  }

  /// Adds every wall mark's triangles, each mark in the sector its bounds lie in, under the slot's shader and texture
  /// and the wall marks' compiler shader. Each corner is a vertex of its own, as the tool writes them; the face's game
  /// material and smoothing are what the editor leaves unset, here zero.
  ///
  /// # Errors
  ///
  /// Returns an error when a table outgrows its indices.
  pub(crate) fn add_wallmarks(
    &mut self,
    tool: &EditableWallmarkTool,
    sectors: &LevelBuildSectors,
    tables: &mut LevelBuildTables,
  ) -> XrfResult {
    for slot in &tool.slots {
      for mark in &slot.marks {
        let sector: u16 = sectors.calculate(&mark.bounds.center, mark.bounds.radius);
        let material: u16 = tables.get_material(
          &slot.shader,
          Some(Self::WALLMARK_COMPILER_SHADER),
          &slot.texture,
          sector,
        )?;

        for triangle in mark.vertices.as_chunks::<3>().0 {
          let first: u32 = u32::try_from(self.vertices.len())
            .map_err(|_| XrfError::new_invalid_error("A build holds more vertices than its u32 indices number"))?;

          self.vertices.extend(triangle.iter().map(|vertex| vertex.position));
          self.faces.push(BuildPrjFace {
            vertices: [first, first + 1, first + 2],
            texture_coordinates: [0, 1, 2].map(|corner| Vector2d::new(triangle[corner].u, triangle[corner].v)),
            material,
            material_padding: 0,
            game_material: 0,
          });
          self.smoothing_groups.push(0);
        }
      }
    }

    Ok(())
  }

  /// `CalcArea` (`BuilderRemote.cpp:509`): Heron's formula over the edges, which is not a number for a triangle
  /// rounding makes impossible.
  fn get_area([a, b, c]: &[Vector3d; 3]) -> f32 {
    let (first, second, third) = (a.distance(b), a.distance(c), b.distance(c));
    let half: f32 = (first + second + third) / 2.0;

    (half * (half - first) * (half - second) * (half - third)).sqrt()
  }
}
