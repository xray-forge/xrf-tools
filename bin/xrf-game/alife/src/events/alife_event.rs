use xrf_game_hooks::ObjectEvent;
use xrf_game_objects::ObjectId;

use crate::switch::alife_switch_direction::AlifeSwitchDirection;

/// What A-Life tells of a step, after it: told to its event hook (`alife.on_switched`), then queued for presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlifeEvent {
  /// Object `id` went online or offline, with what holds it or the squad it is in, or alone (X-Ray's
  /// `on_switch_online` and `on_switch_offline`).
  Switched {
    id: ObjectId,
    direction: AlifeSwitchDirection,
  },
}

impl ObjectEvent for AlifeEvent {
  fn get_object(&self) -> ObjectId {
    match self {
      Self::Switched { id, .. } => *id,
    }
  }
}
