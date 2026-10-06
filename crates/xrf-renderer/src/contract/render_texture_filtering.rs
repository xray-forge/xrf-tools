use serde::{Deserialize, Serialize};

/// How a level's surfaces sample their textures: trilinear, or anisotropic at up to a number of samples along the
/// direction a texture is seen squashed, which keeps ground and walls seen at grazing angles sharp.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderTextureFiltering {
  Trilinear,
  Anisotropic2,
  Anisotropic4,
  #[default]
  Anisotropic8,
  Anisotropic16,
}

impl RenderTextureFiltering {
  /// The sampler's anisotropy clamp: one for trilinear.
  pub fn get_anisotropy(self) -> u16 {
    match self {
      Self::Trilinear => 1,
      Self::Anisotropic2 => 2,
      Self::Anisotropic4 => 4,
      Self::Anisotropic8 => 8,
      Self::Anisotropic16 => 16,
    }
  }
}
