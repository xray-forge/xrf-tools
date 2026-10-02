use serde::{Deserialize, Serialize};

/// The gestures a viewport is told about, named as the browser names them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum RenderInputKind {
  #[serde(rename = "contextmenu")]
  ContextMenu,
  #[serde(rename = "pointercancel")]
  PointerCancel,
  #[serde(rename = "pointerdown")]
  PointerDown,
  #[default]
  #[serde(rename = "pointermove")]
  PointerMove,
  #[serde(rename = "pointerup")]
  PointerUp,
  #[serde(rename = "wheel")]
  Wheel,
  #[serde(rename = "keydown")]
  KeyDown,
  #[serde(rename = "keyup")]
  KeyUp,
  /// The viewport lost focus, so no key it heard go down is still held.
  #[serde(rename = "blur")]
  Blur,
}
