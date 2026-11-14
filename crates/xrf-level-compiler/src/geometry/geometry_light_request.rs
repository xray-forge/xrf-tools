use xrf_level::{BuildCformFile, LevelLightsFile};

use crate::compiler_shaders::CompilerShaders;
use crate::geometry::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::geometry_level_request::GeometryLevelRequest;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_normals::GeometryNormals;
use crate::light::light_bake_request::LightBakeRequest;

/// What [`crate::geometry::geometry_light::GeometryLight::bake`] lights: a compile's request and the geometry it has
/// made so far.
#[derive(Clone, Copy)]
pub(crate) struct GeometryLightRequest<'a> {
  pub(crate) level: &'a GeometryLevelRequest<'a>,
  pub(crate) bake: LightBakeRequest<'a>,
  /// The welded, optimised and tessellated mesh.
  pub(crate) mesh: &'a GeometryMesh,
  pub(crate) normals: &'a GeometryNormals,
  pub(crate) coordinates: &'a GeometryBaseCoordinates,
  pub(crate) shaders: &'a CompilerShaders,
  /// `build.cform`, which every shadow ray is traced through.
  pub(crate) ray_model: &'a BuildCformFile,
  /// `build.lights`, the lights the level is lit by.
  pub(crate) lights: &'a LevelLightsFile,
}
