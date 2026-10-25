use std::path::PathBuf;

use serde::Deserialize;

use crate::parts::LevelMapArea;
use crate::viewer::map::annotations::LevelAnnotationKind;

/// The annotations asked of a level map: its PNG in the screenshots folder, beside which they are written as an SVG,
/// the ground it covers, what of them is drawn, and whether every volume is named.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapAnnotationsWriteRequest {
  pub path: PathBuf,
  pub area: LevelMapArea,
  pub kinds: Vec<LevelAnnotationKind>,
  pub is_labelled: bool,
}
