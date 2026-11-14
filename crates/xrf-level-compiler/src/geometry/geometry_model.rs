use xrf_math::Vector3d;

use crate::geometry::geometry_visual::GeometryVisual;

/// One multi-use model's visuals, built once and placed by each of its references (`calc_ogf`,
/// `xrLC/xrMU_Model_Calc_ogf.cpp:34`).
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryModel {
  /// A visual a rendering material, in material order.
  pub visuals: Vec<GeometryVisual>,
  /// Where the wind terms are measured from (`export_geometry`): the middle of the model's box across, its floor up.
  pub wind_base: Vector3d,
  /// The model's height, which the wind terms are measured in.
  pub wind_height: f32,
  /// Its vertices' places once the normals pass split them, which a LOD's light is gathered from.
  pub positions: Vec<Vector3d>,
}
