use xrf_level::BuildPrjFile;

use crate::compiler_shaders::CompilerShaders;
use crate::compiler_texture::CompilerTexture;
use crate::geometry::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_normals::GeometryNormals;
use crate::geometry::geometry_splits::GeometrySplits;
use crate::geometry::geometry_tangents::GeometryTangents;

/// What [`crate::GeometryVisuals::build`] converts: a level's mesh as the earlier phases left it, its splits, and the
/// project's tables.
#[derive(Clone, Copy, Debug)]
pub struct GeometryVisualsRequest<'a> {
  pub mesh: &'a GeometryMesh,
  pub normals: &'a GeometryNormals,
  pub coordinates: &'a GeometryBaseCoordinates,
  pub tangents: &'a GeometryTangents,
  pub splits: &'a GeometrySplits,
  pub project: &'a BuildPrjFile,
  pub shaders: &'a CompilerShaders,
  /// Each `build.prj` texture as xrLC loads it.
  pub textures: &'a [CompilerTexture],
}
