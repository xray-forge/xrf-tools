use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjFile, BuildPrjMuModel, LevelCformFace, LevelCformGeometry};
use xrf_math::{Aabb, Vector3d};

use crate::compiler_shaders::CompilerShaders;
use crate::geometry::geometry_collector::GeometryCollector;
use crate::geometry::geometry_face::GeometryFace;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_placement::GeometryPlacement;

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
      let model: &BuildPrjMuModel = GeometryPlacement::get_model(project, index, reference)?;

      for (number, face) in model.faces.iter().enumerate() {
        if !shaders.require(model, number, face)?.is_collidable() {
          continue;
        }

        faces.push(Self::pack(
          &mut collector,
          &GeometryPlacement::place(model, number, face, reference)?,
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
}
