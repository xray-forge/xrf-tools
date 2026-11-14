use xrf_level::{BuildCformFile, LevelLightsFile};

use crate::geometry::level::geometry_level_request::GeometryLevelRequest;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::surface::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::input::compiler_shaders::CompilerShaders;
use crate::ray::ray_tracing::RayTracing;

/// What [`crate::light::bake::light_baker::LightBaker::bake`] lights: a compile's request and the geometry it has
/// made so far.
#[derive(Clone, Copy)]
pub(crate) struct LightBakeInput<'a> {
  pub(crate) level: &'a GeometryLevelRequest<'a>,
  /// What traces the lightmaps' and implicit maps' shadow rays.
  pub(crate) tracing: RayTracing<'a>,
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
