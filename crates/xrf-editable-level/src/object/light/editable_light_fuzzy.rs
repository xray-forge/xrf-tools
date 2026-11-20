use xrf_math::Vector3d;

/// The points a fuzzy point light spreads over, `CLight::SFuzzyData`: the build makes a static light of each, sharing
/// the light's energy (`SceneBuilder::BuildPointLight`).
#[derive(Clone, Debug, PartialEq)]
pub struct EditableLightFuzzy {
  /// `EShapeType`: 0 a sphere, 1 a box, the volume new points are spread through.
  pub shape: u8,
  pub sphere_radius: f32,
  pub box_dimensions: Vector3d,
  /// The points, in the light's own space.
  pub points: Vec<Vector3d>,
}
