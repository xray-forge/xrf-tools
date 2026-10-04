import { describe, expect, it, jest } from "@jest/globals";

import { listenRenderClicks, RENDER_CLICK_SLOP, RENDER_DOUBLE_CLICK_TIME } from "@/core/render/lib/frame/render-clicks";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";

function createElement(): HTMLElement {
  const element: HTMLElement = document.createElement("canvas");

  element.getBoundingClientRect = () => ({ left: 100, top: 50 }) as DOMRect;

  return element;
}

function press(
  element: HTMLElement,
  type: string,
  x: number,
  y: number,
  init: MouseEventInit = {},
  timeStamp: number = 0
): void {
  const event: MouseEvent = new MouseEvent(type, { button: 0, clientX: x, clientY: y, ...init });

  Object.defineProperties(event, {
    isPrimary: { value: true },
    pointerId: { value: 1 },
    timeStamp: { value: timeStamp },
  });
  element.dispatchEvent(event);
}

describe("listenRenderClicks", () => {
  it("hears a press let go where it went down, in the element's own pixels", () => {
    const element: HTMLElement = createElement();
    const onClick = jest.fn<(point: IRenderViewPoint) => void>();

    listenRenderClicks(element, onClick);
    press(element, "pointerdown", 140, 90);
    press(element, "pointerup", 141, 91);

    expect(onClick.mock.calls).toEqual([[{ x: 41, y: 41 }]]);
  });

  // The fly camera looks around by dragging the main button, which must never pick what it ends over.
  it("hears nothing of a drag, another button or a cancelled press, nor anything once stopped", () => {
    const element: HTMLElement = createElement();
    const onClick = jest.fn<(point: IRenderViewPoint) => void>();
    const stop: () => void = listenRenderClicks(element, onClick);

    press(element, "pointerdown", 140, 90);
    press(element, "pointerup", 140 + RENDER_CLICK_SLOP + 1, 90);
    press(element, "pointerdown", 140, 90, { button: 2 });
    press(element, "pointerup", 140, 90, { button: 2 });
    press(element, "pointerdown", 140, 90);
    press(element, "pointercancel", 140, 90);
    press(element, "pointerup", 140, 90);
    stop();
    press(element, "pointerdown", 140, 90);
    press(element, "pointerup", 140, 90);

    expect(onClick).not.toHaveBeenCalled();
  });

  // Each pick opens a panel; a double click picking twice moved the viewport under a drag started right after it.
  it("hears a double click as one click, and a click after it as another", () => {
    const element: HTMLElement = createElement();
    const onClick = jest.fn<(point: IRenderViewPoint) => void>();

    listenRenderClicks(element, onClick);
    press(element, "pointerdown", 140, 90, {}, 0);
    press(element, "pointerup", 140, 90, {}, 50);
    press(element, "pointerdown", 140, 90, {}, 150);
    press(element, "pointerup", 140, 90, {}, 200);

    expect(onClick).toHaveBeenCalledTimes(1);

    press(element, "pointerdown", 140, 90, {}, 200 + RENDER_DOUBLE_CLICK_TIME);
    press(element, "pointerup", 140, 90, {}, 250 + RENDER_DOUBLE_CLICK_TIME);

    expect(onClick).toHaveBeenCalledTimes(2);
  });
});
