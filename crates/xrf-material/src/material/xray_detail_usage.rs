use serde::Serialize;
use xrf_thm::ThmDetailUsage;

/// How a detail texture is applied, from the two texture param flags (`TextureDescrManager.cpp`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayDetailUsage {
  Diffuse,
  Bump,
  DiffuseAndBump,
}

impl From<ThmDetailUsage> for XrayDetailUsage {
  fn from(usage: ThmDetailUsage) -> Self {
    match usage {
      ThmDetailUsage::Diffuse => Self::Diffuse,
      ThmDetailUsage::Bump => Self::Bump,
      ThmDetailUsage::DiffuseAndBump => Self::DiffuseAndBump,
    }
  }
}
