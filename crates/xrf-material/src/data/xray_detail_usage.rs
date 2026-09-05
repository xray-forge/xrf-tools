use serde::Serialize;
use xrf_db::ThmDetailUsage;

/// How a detail texture is applied, from the two texture param flags (`TextureDescrManager.cpp`).
///
/// A bump detail brings its own bump and bump# pair, looked up through the detail texture's own descriptor
/// (`uber_deffer.cpp`); that pair is not resolved here.
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
