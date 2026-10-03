use serde::Serialize;

use crate::contract::render_load_failure::RenderLoadFailure;
use crate::contract::render_sector_skip::RenderSectorSkip;

/// What a viewport's level could not draw the way the level asked: drawables left out of the sectors resident, sectors
/// that could not be read, and spawned models that could not be.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLevelProblems {
  pub skipped: Vec<RenderSectorSkip>,
  pub sectors: Vec<RenderLoadFailure>,
  pub models: Vec<RenderLoadFailure>,
}
