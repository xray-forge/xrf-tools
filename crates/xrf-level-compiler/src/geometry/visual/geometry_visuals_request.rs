use xrf_level::BuildPrjFile;

use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::split::geometry_splits::GeometrySplits;
use crate::geometry::surface::geometry_base_coordinates::GeometryBaseCoordinates;
use crate::geometry::surface::geometry_normals::GeometryNormals;
use crate::geometry::surface::geometry_tangents::GeometryTangents;
use crate::input::compiler_shaders::CompilerShaders;
use crate::input::compiler_texture::CompilerTexture;
use crate::light::lightmap::atlas::lightmap_faces::LightmapFaces;
use crate::light::point::light_color::LightColor;

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
  /// Each split vertex's light ([`crate::LightVertices`]), by index; none for the draft light.
  pub lights: Option<&'a [LightColor]>,
  /// Each face's lightmap page and coordinates, as the light baker packed its charts (`LightmapAtlas`); none for the
  /// draft lightmap.
  pub lightmaps: Option<&'a LightmapFaces>,
}
