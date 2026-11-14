use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjFace, BuildPrjFile, BuildPrjMuModel, BuildPrjMuReference, LevelCformFace, LevelCformGeometry};
use xrf_math::{Aabb, Matrix4x4, Vector3d};

use crate::compiler_shaders::CompilerShaders;
use crate::geometry::geometry_collector::GeometryCollector;
use crate::geometry::geometry_face::GeometryFace;
use crate::geometry::geometry_mesh::GeometryMesh;

/// A level's collision form, `level.cform`, as `CBuild::BuildCForm` builds it (`xrLC/xrBuildCForm.cpp:83`) at draft
/// quality: no simplification.
///
/// The static faces whose compiler shader collides, in the mesh's order, then each multi-use reference's, in
/// `build.prj`'s order, its model's colliding faces in theirs placed by its transform (`export_cform_game`,
/// `xrLC_Light/xrMU_Model_Reference.cpp:40`). Each corner is packed by a `GeometryCollector` over the box of the
/// static faces' corners. A face keeps its game material, cut to the 14 bits the form holds, and its sector: its
/// material's for a static face, the reference's for a model's. The two flag bits, which xrLC leaves uninitialised and
/// the engine sets from the game material on load, are written clear.
pub struct GeometryCollision;

impl GeometryCollision {
  /// The collision form of `mesh` and `project`'s multi-use references, by `shaders`.
  ///
  /// # Errors
  ///
  /// Returns an error where xrLC would read past a table: a reference naming a model the project lacks, a model face
  /// naming a vertex the model lacks or a material of no compiler shader; and for a level with no static face to
  /// collide, whose form has no box to pack in.
  pub fn build(
    mesh: &GeometryMesh,
    project: &BuildPrjFile,
    shaders: &CompilerShaders,
  ) -> XrfResult<LevelCformGeometry> {
    let collides = |material: u16| shaders.get(material).is_some_and(|shader| shader.is_collidable());
    let statics: Vec<&GeometryFace> = mesh.faces.iter().filter(|face| collides(face.material)).collect();
    let bounds: Aabb = Aabb::from_points(statics.iter().flat_map(|face| face.get_corners(&mesh.vertices)))
      .ok_or_else(|| XrfError::new_invalid_error("The level has no static face to collide with"))?;
    let mut collector: GeometryCollector = GeometryCollector::new(&bounds);
    let mut faces: Vec<LevelCformFace> = Vec::with_capacity(statics.len());
    let sector_of = |material: u16| {
      project
        .materials
        .get(usize::from(material))
        .map_or(0, |material| material.sector)
    };

    for face in statics {
      faces.push(Self::pack(
        &mut collector,
        &face.get_corners(&mesh.vertices),
        face.game_material,
        sector_of(face.material),
      ));
    }

    for (index, reference) in project.mu_references.iter().enumerate() {
      let model: &BuildPrjMuModel = project.mu_models.get(reference.model as usize).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "build.prj multi-use reference {index} names model {} of {}",
          reference.model,
          project.mu_models.len()
        ))
      })?;

      for (number, face) in model.faces.iter().enumerate() {
        if shaders.get(face.material).is_none() {
          return Err(XrfError::new_invalid_error(format!(
            "Multi-use model {} face {number} is of material {}, which has no compiler shader",
            model.name, face.material
          )));
        }

        if !collides(face.material) {
          continue;
        }

        let corners: [Vector3d; 3] = Self::place(model, face, reference).ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "Multi-use model {} face {number} names a vertex it lacks",
            model.name
          ))
        })?;

        faces.push(Self::pack(
          &mut collector,
          &corners,
          face.game_material,
          reference.sector,
        ));
      }
    }

    LevelCformGeometry::new(collector.into_vertices(), faces)
  }

  /// One face of the form, its corners packed (`CollectorPacked::add_face`).
  fn pack(
    collector: &mut GeometryCollector,
    corners: &[Vector3d; 3],
    game_material: u32,
    sector: u16,
  ) -> LevelCformFace {
    LevelCformFace {
      vertices: corners.map(|corner| collector.pack(&corner)),
      material: (game_material & LevelCformFace::MATERIAL_MASK) as u16,
      is_shadow_suppressed: false,
      is_wallmark_suppressed: false,
      sector,
    }
  }

  /// A model face's corners placed by its reference, `Fmatrix::transform_tiny`; `None` for a corner past the model.
  fn place(model: &BuildPrjMuModel, face: &BuildPrjFace, reference: &BuildPrjMuReference) -> Option<[Vector3d; 3]> {
    let corners: [&Vector3d; 3] = [
      model.vertices.get(face.vertices[0] as usize)?,
      model.vertices.get(face.vertices[1] as usize)?,
      model.vertices.get(face.vertices[2] as usize)?,
    ];

    Some(corners.map(|corner| Self::transform(&reference.transform, corner)))
  }

  /// `Fmatrix::transform_tiny`: a row vector by the matrix, summed left to right, the translation last.
  fn transform(matrix: &Matrix4x4, point: &Vector3d) -> Vector3d {
    let m: &[f32; Matrix4x4::ELEMENTS] = &matrix.values;

    Vector3d::new(
      point.x * m[0] + point.y * m[4] + point.z * m[8] + m[12],
      point.x * m[1] + point.y * m[5] + point.z * m[9] + m[13],
      point.x * m[2] + point.y * m[6] + point.z * m[10] + m[14],
    )
  }
}
