use serde::{Deserialize, Serialize};

use crate::contract::render_load_durations::RenderLoadDurations;

/// How far a viewport's level has been read and put on the GPU, sent as `RenderViewportEvent::Load` for the level drawn
/// and answered to a caller polling `describe_load` for the level last asked for.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLoadReport {
  /// Sectors resident on the GPU.
  pub sectors: u32,
  /// Sectors the level has.
  pub sectors_total: u32,
  /// Bytes of the sectors resident, packed.
  pub bytes: u64,
  /// Textures uploaded or given up on.
  pub textures: u32,
  /// Textures the level samples: its scene's, its lights' projectors and its particles'.
  pub textures_total: u32,
  /// Whether everything is resident, so the level draws as it will: every sector, the spawn, grass, lights and
  /// particles read, and every texture settled.
  pub is_ready: bool,
  /// How long each part took to finish, timed from when the level began opening.
  pub durations: RenderLoadDurations,
}
