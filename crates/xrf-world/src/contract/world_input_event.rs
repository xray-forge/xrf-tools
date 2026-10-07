use serde::{Deserialize, Serialize};

use crate::contract::world_input_kind::WorldInputKind;

/// One gesture over a viewport, as much of the browser's event as crosses.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInputEvent {
  pub kind: WorldInputKind,
  pub pointer_id: i32,
  pub is_primary: bool,
  pub button: i32,
  pub buttons: u32,
  /// CSS pixels from the viewport element's left edge.
  pub x: f32,
  /// CSS pixels from the viewport element's top edge.
  pub y: f32,
  pub delta_x: f32,
  pub delta_y: f32,
  pub delta_mode: u32,
  pub alt_key: bool,
  pub ctrl_key: bool,
  pub meta_key: bool,
  pub shift_key: bool,
  /// `KeyboardEvent.code` for a key, empty for anything else: the key's place, so `W` is `W` on azerty too.
  pub code: String,
}
