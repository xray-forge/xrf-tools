use serde::Serialize;
use xrf_visual::SectorSkip;

/// A drawable of a level's sector the packer left out, and why.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSectorSkip {
  pub sector: u32,
  pub skip: SectorSkip,
}
