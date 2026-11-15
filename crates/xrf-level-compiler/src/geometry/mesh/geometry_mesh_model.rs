use xrf_error::{XrfError, XrfResult};
use xrf_level::BuildPrjMuModel;
use xrf_math::Aabb;

use crate::geometry::mesh::geometry_face::GeometryFace;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;

impl GeometryMesh {
  /// A multi-use model's geometry as `xrMU_Model::Load` reads it (`xrLC_Light/xrMU_Model_Load.cpp`): a vertex each of
  /// the model's, none welded, and its faces in order with their smoothing flags.
  ///
  /// # Errors
  ///
  /// Returns an error for a model of no vertex, without a smoothing flag a face, or with a face naming a vertex it
  /// lacks, which xrLC would read past.
  pub fn of_model(model: &BuildPrjMuModel) -> XrfResult<Self> {
    let smoothing: &[u32] = model.smoothing_groups.as_deref().unwrap_or_default();

    if smoothing.len() != model.faces.len() {
      return Err(XrfError::new_invalid_error(format!(
        "Multi-use model {} has {} smoothing flags for {} faces",
        model.name,
        smoothing.len(),
        model.faces.len()
      )));
    }

    if let Some((number, face)) = model.faces.iter().enumerate().find(|(_, face)| {
      face
        .vertices
        .iter()
        .any(|vertex| *vertex as usize >= model.vertices.len())
    }) {
      return Err(XrfError::new_invalid_error(format!(
        "Multi-use model {} face {number} names vertex {:?} of {}",
        model.name,
        face.vertices,
        model.vertices.len()
      )));
    }

    Ok(Self {
      bounds: Aabb::from_points(&model.vertices)
        .ok_or_else(|| XrfError::new_invalid_error(format!("Multi-use model {} has no vertex", model.name)))?,
      vertices: model.vertices.clone(),
      faces: model
        .faces
        .iter()
        .zip(smoothing)
        .enumerate()
        .map(|(source, (face, smoothing))| GeometryFace {
          vertices: face.vertices,
          texture_coordinates: face.texture_coordinates,
          material: face.material,
          game_material: face.game_material,
          smoothing: *smoothing,
          source: source as u32,
        })
        .collect(),
    })
  }
}
