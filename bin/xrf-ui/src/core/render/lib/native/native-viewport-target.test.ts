import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";

import { RenderViewportLayout } from "@/core/ipc/types/xrf-renderer";
import { EWorldInputKind, WorldInputEvent } from "@/core/ipc/types/xrf-world";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { NativeViewportTarget } from "@/core/render/lib/native/native-viewport-target";

interface IViewportSpy {
  layouts: Array<RenderViewportLayout>;
  inputs: Array<WorldInputEvent>;
  viewport: NativeViewport;
}

function mockViewport(): IViewportSpy {
  const layouts: Array<RenderViewportLayout> = [];
  const inputs: Array<WorldInputEvent> = [];
  const viewport = {
    sendInput: (event: WorldInputEvent) => inputs.push(event),
    setLayout: (layout: RenderViewportLayout) => layouts.push(layout),
  } as unknown as NativeViewport;

  return { inputs, layouts, viewport };
}

function mockTree(): { outer: HTMLElement; inner: HTMLElement; element: HTMLElement } {
  const outer: HTMLElement = document.createElement("div");
  const inner: HTMLElement = document.createElement("div");
  const element: HTMLElement = document.createElement("div");

  outer.style.backgroundColor = "rgb(10, 20, 30)";
  inner.style.backgroundColor = "rgb(40, 50, 60)";
  outer.appendChild(inner);
  inner.appendChild(element);
  document.body.appendChild(outer);
  jest
    .spyOn(element, "getBoundingClientRect")
    .mockReturnValue({ bottom: 120, height: 100, left: 10, right: 210, top: 20, width: 200 } as DOMRect);

  return { element, inner, outer };
}

function pointer(type: string, init: MouseEventInit): Event {
  const event: Event = new MouseEvent(type, { bubbles: true, ...init });

  // jsdom has no pointer events: the two fields a mouse event lacks are put on it.
  Object.defineProperties(event, { isPrimary: { value: true }, pointerId: { value: 1 } });

  return event;
}

describe("NativeViewportTarget", () => {
  beforeEach(() => {
    jest.useFakeTimers();
    Object.defineProperty(window, "devicePixelRatio", { configurable: true, value: 1.5 });
  });

  afterEach(() => {
    jest.useRealTimers();
    document.body.innerHTML = "";
  });

  it("opens a hole through every painted ancestor and closes it again", () => {
    const { element, inner, outer } = mockTree();
    const { layouts, viewport } = mockViewport();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport);

    expect(inner.style.background).toBe("transparent");
    expect(outer.style.background).toBe("transparent");
    // The page showed the nearest painted colour around the viewport, which paints no wash.
    expect(layouts[0].backdrop).toEqual({ color: { b: 60, g: 50, r: 40 }, wash: null });

    target.dispose();

    expect(inner.style.backgroundColor).toBe("rgb(40, 50, 60)");
    expect(outer.style.backgroundColor).toBe("rgb(10, 20, 30)");
  });

  it("reports where the element is in device pixels, and only when it moved", () => {
    const { element } = mockTree();
    const { layouts, viewport } = mockViewport();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport);

    jest.advanceTimersByTime(100);

    expect(layouts).toHaveLength(1);
    expect(layouts[0].rect).toEqual({ height: 150, width: 300, x: 15, y: 30 });
    expect(layouts[0].scale).toBe(1.5);

    target.dispose();
  });

  it("merges pointer moves into one a frame, sent before the release after them", () => {
    const { element } = mockTree();
    const { inputs, viewport } = mockViewport();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport);

    element.dispatchEvent(pointer("pointerdown", { button: 0, clientX: 20, clientY: 30 }));
    element.dispatchEvent(pointer("pointermove", { clientX: 25, clientY: 30 }));
    element.dispatchEvent(pointer("pointermove", { clientX: 40, clientY: 35 }));

    expect(inputs.map((it) => it.kind)).toEqual([EWorldInputKind.POINTER_DOWN]);
    expect(element.style.cursor).toBe("grabbing");

    element.dispatchEvent(pointer("pointerup", { button: 0, clientX: 40, clientY: 35 }));

    expect(inputs.map((it) => it.kind)).toEqual([
      EWorldInputKind.POINTER_DOWN,
      EWorldInputKind.POINTER_MOVE,
      EWorldInputKind.POINTER_UP,
    ]);
    // In css pixels from the element's own corner.
    expect([inputs[1].x, inputs[1].y]).toEqual([30, 15]);
    expect(element.style.cursor).toBe("");

    target.dispose();
  });

  it("forwards keys once while held", () => {
    const { element } = mockTree();
    const { inputs, viewport } = mockViewport();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport);

    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));
    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW", repeat: true }));
    element.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyW" }));

    expect(inputs.map((it) => [it.kind, it.code])).toEqual([
      [EWorldInputKind.KEY_DOWN, "KeyW"],
      [EWorldInputKind.KEY_UP, "KeyW"],
    ]);

    target.dispose();
  });
});
