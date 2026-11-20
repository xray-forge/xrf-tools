use crate::light::bake::light_bake::LightBake;
use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::lightmap::atlas::lightmap_faces::LightmapFaces;
use crate::light::lightmap::atlas::lightmap_images::LightmapImages;
use crate::light::point::light_color::LightColor;
use crate::light::vertex::light_models::LightModels;

/// A level's baked light ([`crate::light::bake::light_baker::LightBaker`]): the vertices', models' and placements',
/// the lightmap pages and each face's place on them, and the implicitly lit textures' maps.
pub(crate) struct LightBakeResult {
  /// Each split vertex's light.
  pub(crate) vertices: Vec<LightColor>,
  pub(crate) models: LightModels,
  /// Each face's lightmap page and coordinates.
  pub(crate) faces: LightmapFaces,
  /// Each lightmap page's two textures
  /// ([`crate::light::lightmap::atlas::lightmap_atlas::LightmapAtlas::compose_all`]).
  pub(crate) lightmaps: LightmapImages,
  /// Each texture lit implicitly, such as terrain's; none at draft build quality.
  pub(crate) implicit_maps: Vec<ImplicitMap>,
  pub(crate) bake: LightBake,
}
