use std::collections::HashSet;

use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildCformFace, BuildCformFile, BuildCformRaycastFace, BuildPrjFile, BuildPrjMuModel};
use xrf_math::{Vector2d, Vector3d};

use crate::compiler_shaders::CompilerShaders;
use crate::geometry::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::geometry_collector::GeometryCollector;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_normals::GeometryNormals;
use crate::geometry::geometry_placement::GeometryPlacement;

/// The ray model the light, detail and AI compilers query, `build.cform`, as `CBuild::BuildRapid` builds and saves it
/// (`xrLC/xrBuildRapidModel.cpp:41`).
///
/// The static faces whose compiler shader casts a shadow, in the mesh's order, then each multi-use reference's, in
/// `build.prj`'s order, its model's shadow-casting faces in theirs placed by its transform (`export_cform_rcast`). A
/// static face with the same three corners as one already taken is left out, its corners as the smoothing split leaves
/// them, so a double-sided face's back stays apart from its front. A model's faces are all taken: the source drops a
/// model face repeating another's corners too, but SDK 0.7's xrLC keeps every one, and its runs are the reference.
/// Each corner is packed by a `GeometryCollector` over the scene's box, which
/// is also the form's. A face's record holds its `build.prj` material, its game material and its base texture
/// coordinates, a static face's as the tangent-basis pass leaves them (`GeometryBaseCoordinates`); its handle, xrLC's face address that no reader uses, is its index here.
pub struct GeometryRayModel;

impl GeometryRayModel {
  /// The ray model of `mesh`, split as `normals` with `coordinates`, and `project`'s multi-use references, by
  /// `shaders`.
  ///
  /// # Errors
  ///
  /// Returns an error when `normals` or `coordinates` is not of `mesh`'s faces, and where xrLC would read past a table: a reference
  /// naming a model the project lacks, a model face naming a vertex the model lacks or a material of no compiler
  /// shader.
  pub fn build(
    mesh: &GeometryMesh,
    normals: &GeometryNormals,
    coordinates: &GeometryBaseCoordinates,
    project: &BuildPrjFile,
    shaders: &CompilerShaders,
  ) -> XrfResult<BuildCformFile> {
    if normals.faces.len() != mesh.faces.len() || coordinates.faces.len() != mesh.faces.len() {
      return Err(XrfError::new_invalid_error(format!(
        "The smoothing split holds {} faces and the base coordinates {} of the mesh's {}",
        normals.faces.len(),
        coordinates.faces.len(),
        mesh.faces.len()
      )));
    }

    let mut collector: GeometryCollector = GeometryCollector::new(&mesh.bounds);
    let mut faces: Vec<BuildCformFace> = Vec::new();
    let mut raycast_faces: Vec<BuildCformRaycastFace> = Vec::new();
    let mut taken: HashSet<[u32; 3]> = HashSet::new();
    let casts_shadow = |material: u16| shaders.get(material).is_some_and(|shader| shader.casts_shadow());

    for ((face, corners), coordinates) in mesh.faces.iter().zip(&normals.faces).zip(&coordinates.faces) {
      if casts_shadow(face.material) && taken.insert(Self::get_key(corners)) {
        Self::push(
          &mut collector,
          &mut faces,
          &mut raycast_faces,
          &face.get_corners(&mesh.vertices),
          Self::get_record(face.material, face.game_material, *coordinates),
        );
      }
    }

    for (index, reference) in project.mu_references.iter().enumerate() {
      let model: &BuildPrjMuModel = GeometryPlacement::get_model(project, index, reference)?;

      for (number, face) in model.faces.iter().enumerate() {
        if !shaders.require(model, number, face)?.casts_shadow() {
          continue;
        }

        Self::push(
          &mut collector,
          &mut faces,
          &mut raycast_faces,
          &GeometryPlacement::place(model, number, face, reference)?,
          Self::get_record(face.material, face.game_material, face.texture_coordinates),
        );
      }
    }

    Ok(BuildCformFile {
      version: BuildCformFile::CURRENT_VERSION,
      bounds: mesh.bounds,
      vertices: collector.into_vertices(),
      faces,
      raycast_faces,
    })
  }

  /// A face's corners in any order, as `FaceEqual` takes all six.
  fn get_key(corners: &[u32; 3]) -> [u32; 3] {
    let mut key: [u32; 3] = *corners;

    key.sort_unstable();
    key
  }

  /// A face's record, `b_rc_face`, its padding clear.
  fn get_record(material: u16, game_material: u32, texture_coordinates: [Vector2d; 3]) -> BuildCformRaycastFace {
    BuildCformRaycastFace {
      material,
      material_padding: 0,
      game_material,
      texture_coordinates,
      reserved: 0,
      trailing_padding: 0,
    }
  }

  /// One face of the form, its corners packed (`CollectorPacked::add_face_D`), and its record.
  fn push(
    collector: &mut GeometryCollector,
    faces: &mut Vec<BuildCformFace>,
    raycast_faces: &mut Vec<BuildCformRaycastFace>,
    corners: &[Vector3d; 3],
    record: BuildCformRaycastFace,
  ) {
    faces.push(BuildCformFace {
      vertices: corners.map(|corner| collector.pack(&corner)),
      face_handle: faces.len() as u32,
    });
    raycast_faces.push(record);
  }
}
