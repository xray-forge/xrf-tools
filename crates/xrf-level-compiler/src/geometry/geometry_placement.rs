use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjFace, BuildPrjFile, BuildPrjMuModel, BuildPrjMuReference};
use xrf_math::{Matrix4x4, Vector3d};

/// Multi-use models' faces placed in the level by their references, as both collision forms take them
/// (`xrLC_Light/xrMU_Model_Reference.cpp`).
pub(crate) struct GeometryPlacement;

impl GeometryPlacement {
  /// The model reference `index` of `project` places.
  ///
  /// # Errors
  ///
  /// Returns an error for a model the project lacks, which xrLC would read past.
  pub(crate) fn get_model<'a>(
    project: &'a BuildPrjFile,
    index: usize,
    reference: &BuildPrjMuReference,
  ) -> XrfResult<&'a BuildPrjMuModel> {
    project.mu_models.get(reference.model as usize).ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "build.prj multi-use reference {index} names model {} of {}",
        reference.model,
        project.mu_models.len()
      ))
    })
  }

  /// Face `number` of `model` placed by `reference`, each corner by `Fmatrix::transform_tiny`.
  ///
  /// # Errors
  ///
  /// Returns an error for a corner past the model's vertices.
  pub(crate) fn place(
    model: &BuildPrjMuModel,
    number: usize,
    face: &BuildPrjFace,
    reference: &BuildPrjMuReference,
  ) -> XrfResult<[Vector3d; 3]> {
    let corner = |index: u32| {
      model.vertices.get(index as usize).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Multi-use model {} face {number} names vertex {index} of {}",
          model.name,
          model.vertices.len()
        ))
      })
    };

    Ok([
      Self::transform(&reference.transform, corner(face.vertices[0])?),
      Self::transform(&reference.transform, corner(face.vertices[1])?),
      Self::transform(&reference.transform, corner(face.vertices[2])?),
    ])
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
