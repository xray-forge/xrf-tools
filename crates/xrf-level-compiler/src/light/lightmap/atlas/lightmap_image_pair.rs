/// One lightmap page's two textures, their texels row by row: `lmap#N_1`, the static light with the sun in alpha, and
/// `lmap#N_2`, the sun in every colour with the hemisphere in alpha (`lm_layer::Pack`, `Pack_hemi`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LightmapImagePair {
  pub(crate) first: Vec<[u8; 4]>,
  pub(crate) second: Vec<[u8; 4]>,
}
