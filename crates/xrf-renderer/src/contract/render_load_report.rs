use serde::{Deserialize, Serialize};

/// How far a viewport's scene has been read and put on the GPU.
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
  /// Textures the scene names.
  pub textures_total: u32,
  /// Whether everything is resident, so the scene draws as it will.
  pub is_ready: bool,
}
