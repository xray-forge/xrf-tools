/** The gestures a scene on another thread is told about, named as the browser names them. */
export enum ERenderInput {
  CONTEXT_MENU = "contextmenu",
  POINTER_CANCEL = "pointercancel",
  POINTER_DOWN = "pointerdown",
  POINTER_MOVE = "pointermove",
  POINTER_UP = "pointerup",
  WHEEL = "wheel",
  KEY_DOWN = "keydown",
  KEY_UP = "keyup",
  /** The canvas lost focus, so no key it heard go down is still held. */
  BLUR = "blur",
}
