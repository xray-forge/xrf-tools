use crate::light::lightmap::atlas::lightmap_image_pair::LightmapImagePair;

/// Every lightmap page's textures, each holding `side` by `side` texels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LightmapImages {
  /// A page's side in texels.
  pub(crate) side: u32,
  /// Each page's pair, in page order.
  pub(crate) pages: Vec<LightmapImagePair>,
}
