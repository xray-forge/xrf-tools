use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_app_core::types::TauriResult;

use crate::viewer::annotations::level_annotation_kind::LevelAnnotationKind;

/// The ground a level map covers, engine space: its least and greatest `x` and `z`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapAnnotationsArea {
  pub min_x: f32,
  pub min_z: f32,
  pub max_x: f32,
  pub max_z: f32,
}

impl MapAnnotationsArea {
  /// Refuses an area spanning no ground, which places no point on a map.
  ///
  /// # Errors
  ///
  /// Returns an error naming the corners when either least edge is not below its greatest.
  pub fn require_ground(&self) -> TauriResult {
    let Self {
      min_x,
      min_z,
      max_x,
      max_z,
    } = *self;

    if min_x < max_x && min_z < max_z {
      Ok(())
    } else {
      Err(format!(
        "A level map's area spans no ground: {min_x}, {min_z}, {max_x}, {max_z}"
      ))
    }
  }
}

/// The annotations asked of a level map: its PNG in the screenshots folder, beside which they are written as an SVG,
/// the ground it covers, what of them is drawn, and whether every volume is named.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapAnnotationsRequest {
  pub path: PathBuf,
  pub area: MapAnnotationsArea,
  pub kinds: Vec<LevelAnnotationKind>,
  pub is_labelled: bool,
}
