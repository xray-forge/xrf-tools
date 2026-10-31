use std::path::PathBuf;

use serde::Deserialize;

use crate::objects::LevelObjectKind;
use crate::parts::LevelMapArea;

/// The annotations asked of a level map: its PNG in the screenshots folder, beside which they are written as an SVG,
/// the ground it covers, the kinds of level object drawn, and whether every object is named.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapAnnotationsWriteRequest {
  pub path: PathBuf,
  pub area: LevelMapArea,
  pub kinds: Vec<LevelObjectKind>,
  pub is_labelled: bool,
}
