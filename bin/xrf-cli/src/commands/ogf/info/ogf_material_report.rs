use serde::Serialize;
use xrf_ogf::OgfTextureChunk;

/// The texture and shader a visual or one of its children is dressed with.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OgfMaterialReport {
  shader: String,
  texture: String,
}

impl OgfMaterialReport {
  pub(crate) fn new(texture: &OgfTextureChunk) -> Self {
    Self {
      shader: texture.shader_name.clone(),
      texture: texture.texture_name.clone(),
    }
  }
}
