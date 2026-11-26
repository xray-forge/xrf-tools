use std::path::PathBuf;

use serde::Deserialize;
use xrf_level_source::{LevelMapArea, LevelObjectKind};

/// Annotations asked of a level map: `path` is its PNG in the screenshots folder, the SVG is written beside it, `area`
/// is the ground it covers, `kinds` the object kinds drawn, and `is_labelled` names every object.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MapAnnotationsWriteRequest {
  pub path: PathBuf,
  pub area: LevelMapArea,
  pub kinds: Vec<LevelObjectKind>,
  pub is_labelled: bool,
}
