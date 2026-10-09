use serde::{Deserialize, Serialize};

use crate::contract::world_drag_end_reason::WorldDragEndReason;
use crate::contract::world_input_modifiers::WorldInputModifiers;

/// What the person does over a viewport: the drags its page told from clicks, the hover, the wheel and the keys.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WorldInputEvent {
  /// A press travelled far enough to be a drag; `button` is the `PointerEvent.button` pressed, `0` the main one.
  DragStart {
    button: i32,
    modifiers: WorldInputModifiers,
  },
  /// How far the drag moved since its last move: CSS pixels, or the mouse's own counts while the pointer is locked.
  DragMove { dx: f32, dy: f32 },
  /// The drag is over.
  DragEnd { reason: WorldDragEndReason },
  /// Where the pointer rests while nothing drags, in CSS pixels from the viewport element's top left corner.
  Hover { x: f32, y: f32 },
  /// The wheel turned, as `WheelEvent` measures it.
  Wheel {
    delta_x: f32,
    delta_y: f32,
    delta_mode: u32,
  },
  /// A key went down; `code` is `KeyboardEvent.code`, the key's place, so `W` is `W` on azerty too.
  KeyDown { code: String },
  /// A key went up.
  KeyUp { code: String },
  /// The secondary button asked for a menu, which the page withholds.
  ContextMenu,
  /// The viewport lost focus, so no key it heard go down is still held.
  Blur,
}
