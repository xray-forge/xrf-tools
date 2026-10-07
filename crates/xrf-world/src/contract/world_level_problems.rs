use serde::Serialize;
use xrf_renderer::RenderLoadFailure;

use crate::contract::world_sector_skip::WorldSectorSkip;

/// What a viewport's level could not draw the way the level asked: drawables left out of the sectors resident, sectors
/// that could not be read, and spawned models that could not be.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldLevelProblems {
  pub skipped: Vec<WorldSectorSkip>,
  pub sectors: Vec<RenderLoadFailure>,
  pub models: Vec<RenderLoadFailure>,
}
