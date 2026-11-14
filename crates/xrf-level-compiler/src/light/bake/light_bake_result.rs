use crate::light::bake::light_bake::LightBake;
use crate::light::implicit::implicit_map::ImplicitMap;
use crate::light::lightmap::atlas::lightmap_faces::LightmapFaces;
use crate::light::lightmap::atlas::lightmap_images::LightmapImages;
use crate::light::point::light_color::LightColor;
use crate::light::vertex::light_models::LightModels;

/// A level's baked light, the light step's part of its compile ([`crate::light::bake::light_baker::LightBaker`]):
/// every vertex's, every multi-use model's and placement's, each face's place on the lightmap pages and the pages
/// themselves, and each implicitly lit texture's map.
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
