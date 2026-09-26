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

/**
 * One gesture, as much of it as crosses a thread.
 */
export interface IRenderInputEvent {
  type: ERenderInput;
  pointerId: number;
  pointerType: string;
  isPrimary: boolean;
  button: number;
  buttons: number;
  clientX: number;
  clientY: number;
  deltaX: number;
  deltaY: number;
  deltaMode: number;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  /** `KeyboardEvent.code` for a key, empty for anything else: the key's place, so `W` is `W` on azerty too. */
  code: string;
}

/**
 * Takes the part of a browser event that crosses.
 *
 * @param type - Which gesture it is.
 * @param event - The event as the page received it.
 * @returns What to post.
 */
export function toRenderInputEvent(type: ERenderInput, event: Event): IRenderInputEvent {
  const pointer: Partial<PointerEvent> = event as PointerEvent;
  const wheel: Partial<WheelEvent> = event as WheelEvent;
  const key: Partial<KeyboardEvent> = event as KeyboardEvent;

  return {
    altKey: key.altKey ?? false,
    button: pointer.button ?? 0,
    buttons: pointer.buttons ?? 0,
    clientX: pointer.clientX ?? 0,
    clientY: pointer.clientY ?? 0,
    code: key.code ?? "",
    ctrlKey: key.ctrlKey ?? false,
    deltaMode: wheel.deltaMode ?? 0,
    deltaX: wheel.deltaX ?? 0,
    deltaY: wheel.deltaY ?? 0,
    isPrimary: pointer.isPrimary ?? true,
    metaKey: key.metaKey ?? false,
    pointerId: pointer.pointerId ?? 0,
    pointerType: pointer.pointerType ?? "mouse",
    shiftKey: key.shiftKey ?? false,
    type,
  };
}
