import { Nullable } from "@xrf/types";

import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";

/** Css pixels a press may travel and still be a click, rather than the drag a camera looks around by. */
export const RENDER_CLICK_SLOP: number = 4;

/** Milliseconds within which a second click at the same place is a double click's, which picks nothing again. */
export const RENDER_DOUBLE_CLICK_TIME: number = 500;

/** Where the press being watched went down. */
interface IRenderPress {
  pointerId: number;
  x: number;
  y: number;
}

/**
 * Hears clicks on a drawing surface: the main button pressed and let go over it without the pointer travelling, which
 * tells a click apart from a drag.
 *
 * @param element - What is clicked.
 * @param onClick - Told where, in css pixels from the element's top left corner.
 * @returns What stops hearing.
 */
export function listenRenderClicks(element: HTMLElement, onClick: (point: IRenderViewPoint) => void): () => void {
  let press: Nullable<IRenderPress> = null;
  // The last click heard, so the second of a double click is not heard as another.
  let last: Nullable<{ time: number; x: number; y: number }> = null;

  function onDown(event: PointerEvent): void {
    // The main button of the first pointer down alone, as the camera's controls take it.
    press =
      event.isPrimary && event.button === 0 ? { pointerId: event.pointerId, x: event.clientX, y: event.clientY } : null;
  }

  function onUp(event: PointerEvent): void {
    const pressed: Nullable<IRenderPress> = press;

    press = null;

    if (
      pressed?.pointerId === event.pointerId &&
      event.button === 0 &&
      Math.hypot(event.clientX - pressed.x, event.clientY - pressed.y) <= RENDER_CLICK_SLOP
    ) {
      const isSecond: boolean =
        last !== null &&
        event.timeStamp - last.time <= RENDER_DOUBLE_CLICK_TIME &&
        Math.hypot(event.clientX - last.x, event.clientY - last.y) <= RENDER_CLICK_SLOP;

      last = isSecond ? null : { time: event.timeStamp, x: event.clientX, y: event.clientY };

      if (isSecond) {
        return;
      }

      const { left, top } = element.getBoundingClientRect();

      onClick({ x: event.clientX - left, y: event.clientY - top });
    }
  }

  function onCancel(): void {
    press = null;
  }

  element.addEventListener("pointerdown", onDown);
  element.addEventListener("pointerup", onUp);
  element.addEventListener("pointercancel", onCancel);

  return (): void => {
    element.removeEventListener("pointerdown", onDown);
    element.removeEventListener("pointerup", onUp);
    element.removeEventListener("pointercancel", onCancel);
  };
}
