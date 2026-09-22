/** The gestures a scene on another thread is told about, named as the browser names them. */
export enum ERenderInput {
  CONTEXT_MENU = "contextmenu",
  POINTER_CANCEL = "pointercancel",
  POINTER_DOWN = "pointerdown",
  POINTER_MOVE = "pointermove",
  POINTER_UP = "pointerup",
  WHEEL = "wheel",
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
}

/**
 * Takes the part of a browser event that crosses.
 *
 * @param type - Which gesture it is.
 * @param event - The event as the page received it.
 * @returns What to post.
 */
export function toRenderInputEvent(type: ERenderInput, event: PointerEvent | WheelEvent): IRenderInputEvent {
  const pointer: Partial<PointerEvent> = event as PointerEvent;
  const wheel: Partial<WheelEvent> = event as WheelEvent;

  return {
    altKey: event.altKey,
    button: event.button,
    buttons: event.buttons,
    clientX: event.clientX,
    clientY: event.clientY,
    ctrlKey: event.ctrlKey,
    deltaMode: wheel.deltaMode ?? 0,
    deltaX: wheel.deltaX ?? 0,
    deltaY: wheel.deltaY ?? 0,
    isPrimary: pointer.isPrimary ?? true,
    metaKey: event.metaKey,
    pointerId: pointer.pointerId ?? 0,
    pointerType: pointer.pointerType ?? "mouse",
    shiftKey: event.shiftKey,
    type,
  };
}
