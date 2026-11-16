use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjFace, BuildPrjFile, BuildPrjMuModel, BuildPrjMuReference};
use xrf_math::{Matrix4x4, Vector3d};

use crate::geometry::model::geometry_placement_sum::GeometryPlacementSum;

/// The order each axis of each corner of a placed face sums in: by corner, then by axis.
pub(crate) type GeometryFaceSums = [[GeometryPlacementSum; 3]; 3];

/// Multi-use models' faces and points placed in the level by their references, as the collision forms, the visual tree
/// and the LODs take them (`xrLC_Light/xrMU_Model_Reference.cpp`).
pub(crate) struct GeometryPlacement;

impl GeometryPlacement {
  /// How `xrMU_Model::export_cform_rcast` places a face's corners in the ray model, as SDK 0.7's xrLC sums them:
  /// the first two with the point's last two products first, the third in order.
  pub(crate) const RAY_MODEL_SUMS: GeometryFaceSums = {
    use GeometryPlacementSum::{Ordered, YzFirst};

    [[YzFirst; 3], [YzFirst; 3], [Ordered; 3]]
  };

  /// How `xrMU_Reference::export_cform_game` places a face's corners in the collision form, as SDK 0.7's xrLC sums
  /// them: the first with its last two products first on `x` and its outer two first on `y` and `z`, the others in
  /// order.
  pub(crate) const COLLISION_SUMS: GeometryFaceSums = {
    use GeometryPlacementSum::{Ordered, XzFirst, YzFirst};

    [[YzFirst, XzFirst, XzFirst], [Ordered; 3], [Ordered; 3]]
  };

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

  /// Face `number` of `model` placed by `reference`, each corner by `Fmatrix::transform_tiny` summed as `sums` gives.
  ///
  /// # Errors
  ///
  /// Returns an error for a corner past the model's vertices.
  pub(crate) fn place(
    model: &BuildPrjMuModel,
    number: usize,
    face: &BuildPrjFace,
    reference: &BuildPrjMuReference,
    sums: &GeometryFaceSums,
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
      Self::transform_by(&reference.transform, corner(face.vertices[0])?, &sums[0]),
      Self::transform_by(&reference.transform, corner(face.vertices[1])?, &sums[1]),
      Self::transform_by(&reference.transform, corner(face.vertices[2])?, &sums[2]),
    ])
  }

  /// `Fmatrix::transform_tiny`: a row vector by the matrix, summed left to right, the translation last.
  pub(crate) fn transform(matrix: &Matrix4x4, point: &Vector3d) -> Vector3d {
    Self::transform_by(matrix, point, &[GeometryPlacementSum::Ordered; 3])
  }

  /// `Fmatrix::transform_tiny` with each axis summed as `sums` gives, in single precision.
  pub(crate) fn transform_by(matrix: &Matrix4x4, point: &Vector3d, sums: &[GeometryPlacementSum; 3]) -> Vector3d {
    let m: &[f32; Matrix4x4::ELEMENTS] = &matrix.values;
    let axis = |axis: usize| {
      sums[axis].sum(
        point.x * m[axis],
        point.y * m[axis + 4],
        point.z * m[axis + 8],
        m[axis + 12],
      )
    };

    Vector3d::new(axis(0), axis(1), axis(2))
  }
}
