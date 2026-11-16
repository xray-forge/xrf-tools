use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildCformFace, BuildCformFile, BuildCformRaycastFace, BuildPrjFile, BuildPrjMuModel};
use xrf_math::{Vector2d, Vector3d};

use crate::geometry::collision::geometry_collector::GeometryCollector;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::model::geometry_placement::GeometryPlacement;
use crate::geometry::surface::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::input::compiler_shaders::CompilerShaders;

/// The ray model the light, detail and AI compilers query, `build.cform`, as `CBuild::BuildRapid` builds and saves it
/// (`xrLC/xrBuildRapidModel.cpp`).
///
/// The static faces whose compiler shader casts a shadow, in the mesh's order, then each multi-use reference's, in
/// `build.prj`'s order, its model's shadow-casting faces in theirs placed by its transform (`export_cform_rcast`). A
/// static face with the same three corners as one already taken is left out, its corners as the smoothing split leaves
/// them, so a double-sided face's back stays apart from its front. A model face repeating another of its model is left
/// out alike, its corners as the model's own smoothing split leaves them. Each corner is packed by a
/// `GeometryCollector` over the scene's box, which is also the form's. A face's record holds its `build.prj` material,
/// its game material and its base texture coordinates, a static face's as the tangent-basis pass leaves them
/// (`GeometryBaseCoordinates`); its handle, xrLC's face address that no reader uses, is its index.
pub struct GeometryRayModel;

impl GeometryRayModel {
  /// The ray model of `mesh`, split as `normals` with `coordinates`, and `project`'s multi-use references, by
  /// `shaders`.
  ///
  /// # Errors
  ///
  /// Returns an error when `normals` or `coordinates` is not of `mesh`'s faces, and where xrLC would read past a table:
  /// a reference naming a model the project lacks, a placed model with no vertex or without a smoothing flag a face, a
  /// model face naming a vertex the model lacks or a material of no compiler shader.
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

    Self::collect(mesh, &normals.faces, &coordinates.faces, project, shaders, |model| {
      Ok(GeometryNormals::of(&GeometryMesh::of_model(model)?).faces)
    })
  }

  /// The ray model `xrPhase_AdaptiveHT` builds for its sky term before the smoothing split (`BuildRapid(FALSE)`): as
  /// [`Self::build`], but a face repeating another's corners as the weld leaves them is left out, a double-sided face's
  /// back with it, a model face repeating another's corners as the model is loaded, and each face keeps its base
  /// coordinates as loaded.
  ///
  /// # Errors
  ///
  /// As [`Self::build`], for the references.
  pub fn build_welded(
    mesh: &GeometryMesh,
    project: &BuildPrjFile,
    shaders: &CompilerShaders,
  ) -> XrfResult<BuildCformFile> {
    let corners: Vec<[u32; 3]> = mesh.faces.iter().map(|face| face.vertices).collect();
    let coordinates: Vec<[Vector2d; 3]> = mesh.faces.iter().map(|face| face.texture_coordinates).collect();

    Self::collect(mesh, &corners, &coordinates, project, shaders, |model| {
      Ok(model.faces.iter().map(|face| face.vertices).collect())
    })
  }

  /// Where each of `mesh`'s faces, its corners split as `normals` leaves them, stands in [`Self::build`]'s form: its
  /// index there, none for a face that casts no shadow or repeats one taken before it.
  pub(crate) fn get_static_faces(
    mesh: &GeometryMesh,
    normals: &GeometryNormals,
    shaders: &CompilerShaders,
  ) -> Vec<Option<u32>> {
    Self::get_taken(mesh, &normals.faces, shaders)
  }

  /// Each static face's index in the form of `mesh`'s faces named by `corners`, none for one left out.
  fn get_taken(mesh: &GeometryMesh, corners: &[[u32; 3]], shaders: &CompilerShaders) -> Vec<Option<u32>> {
    let mut count: u32 = 0;

    Self::get_unrepeated(mesh.faces.iter().zip(corners).map(|(face, corners)| {
      (
        corners,
        shaders.get(face.material).is_some_and(|shader| shader.casts_shadow()),
      )
    }))
    .into_iter()
    .map(|is_taken| {
      is_taken.then(|| {
        count += 1;
        count - 1
      })
    })
    .collect()
  }

  /// Whether each face of `model` is taken into the form, its corners named by `corners`: one casting a shadow that
  /// repeats no face taken before it (`xrMU_Model::export_cform_rcast`, which clears its marks for each placement, so
  /// every placement of a model takes the same faces).
  fn get_model_taken(model: &BuildPrjMuModel, corners: &[[u32; 3]], shaders: &CompilerShaders) -> XrfResult<Vec<bool>> {
    let casts_shadow: Vec<bool> = model
      .faces
      .iter()
      .enumerate()
      .map(|(number, face)| Ok(shaders.require(model, number, face)?.casts_shadow()))
      .collect::<XrfResult<_>>()?;

    Ok(Self::get_unrepeated(corners.iter().zip(casts_shadow)))
  }

  /// Whether each face, by its corners and whether it casts a shadow, is taken: one casting a shadow with corners no
  /// face taken before it holds, in any order (`FaceEqual` over the faces marked processed).
  fn get_unrepeated<'a>(faces: impl Iterator<Item = (&'a [u32; 3], bool)>) -> Vec<bool> {
    let mut taken: HashSet<[u32; 3]> = HashSet::new();

    faces
      .map(|(corners, casts_shadow)| casts_shadow && taken.insert(Self::get_key(corners)))
      .collect()
  }

  /// The form of `mesh`'s faces, each named by `corners` for the repeat test and with its `coordinates`, then the
  /// references', each model's faces named for the repeat test by `model_corners`.
  fn collect(
    mesh: &GeometryMesh,
    corners: &[[u32; 3]],
    coordinates: &[[Vector2d; 3]],
    project: &BuildPrjFile,
    shaders: &CompilerShaders,
    model_corners: impl Fn(&BuildPrjMuModel) -> XrfResult<Vec<[u32; 3]>>,
  ) -> XrfResult<BuildCformFile> {
    let mut collector: GeometryCollector = GeometryCollector::new(&mesh.bounds);
    let mut faces: Vec<BuildCformFace> = Vec::new();
    let mut raycast_faces: Vec<BuildCformRaycastFace> = Vec::new();

    for ((face, taken), coordinates) in mesh
      .faces
      .iter()
      .zip(Self::get_taken(mesh, corners, shaders))
      .zip(coordinates)
    {
      if taken.is_some() {
        Self::push(
          &mut collector,
          &mut faces,
          &mut raycast_faces,
          &face.get_corners(&mesh.vertices),
          Self::get_record(face.material, face.game_material, *coordinates),
        );
      }
    }

    let mut model_taken: HashMap<u32, Vec<bool>> = HashMap::new();

    for (index, reference) in project.mu_references.iter().enumerate() {
      let model: &BuildPrjMuModel = GeometryPlacement::get_model(project, index, reference)?;
      let taken: &[bool] = match model_taken.entry(reference.model) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => entry.insert(Self::get_model_taken(model, &model_corners(model)?, shaders)?),
      };

      for (number, face) in model.faces.iter().enumerate() {
        if !taken[number] {
          continue;
        }

        Self::push(
          &mut collector,
          &mut faces,
          &mut raycast_faces,
          &GeometryPlacement::place(model, number, face, reference, &GeometryPlacement::RAY_MODEL_SUMS)?,
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
