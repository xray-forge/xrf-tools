import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderViewportLayout } from "@xrf/ipc/types/xrf-renderer";
import { EWorldCamera, EWorldDragEndReason, EWorldInputEvent, WorldInputEvent } from "@xrf/ipc/types/xrf-world";

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
    getCameraKind: () => EWorldCamera.FLY,
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

    // The page paints itself until the renderer has drawn the window, which shows black while its device starts.
    expect(inner.style.backgroundColor).toBe("rgb(40, 50, 60)");
    expect(outer.style.backgroundColor).toBe("rgb(10, 20, 30)");

    target.openHole();

    expect(inner.style.background).toBe("transparent");
    expect(outer.style.background).toBe("transparent");
    // jsdom lays nothing out, so the nearest painted ancestor fills the window and hides the ones outside it.
    expect(layouts[0].backdrop).toEqual({
      planes: [
        {
          color: { b: 60, g: 50, r: 40 },
          isWashed: false,
          radii: [0, 0, 0, 0],
          rect: { height: 0, width: 0, x: 0, y: 0 },
        },
      ],
      wash: null,
    });

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

  it("clicks on a press let go within the slop, telling the world nothing", () => {
    const { element } = mockTree();
    const { inputs, viewport } = mockViewport();
    const onClick = jest.fn();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport, { onClick });

    element.dispatchEvent(pointer("pointerdown", { button: 0, buttons: 1, clientX: 20, clientY: 30 }));
    element.dispatchEvent(pointer("pointermove", { buttons: 1, clientX: 22, clientY: 31 }));
    element.dispatchEvent(pointer("pointerup", { button: 0, clientX: 22, clientY: 31 }));
    // The second click of a double click picks nothing again.
    element.dispatchEvent(pointer("pointerdown", { button: 0, buttons: 1, clientX: 22, clientY: 31 }));
    element.dispatchEvent(pointer("pointerup", { button: 0, clientX: 22, clientY: 31 }));

    // In css pixels from the element's own corner.
    expect(onClick).toHaveBeenCalledTimes(1);
    expect(onClick).toHaveBeenCalledWith({ isThrough: false, isToggling: false, point: { x: 12, y: 11 } });
    expect(inputs).toEqual([]);

    target.dispose();
  });

  it("drags a press past the slop, its motion merged and sent before its end, and never clicks", () => {
    const { element } = mockTree();
    const { inputs, viewport } = mockViewport();
    const onClick = jest.fn();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport, { onClick });

    element.dispatchEvent(pointer("pointerdown", { button: 0, buttons: 1, clientX: 20, clientY: 30 }));
    element.dispatchEvent(pointer("pointermove", { buttons: 1, clientX: 25, clientY: 30 }));
    element.dispatchEvent(pointer("pointermove", { buttons: 1, clientX: 40, clientY: 35 }));

    expect(element.style.cursor).toBe("grabbing");

    element.dispatchEvent(pointer("pointerup", { button: 0, clientX: 40, clientY: 35 }));

    expect(inputs).toEqual([
      {
        button: 0,
        kind: EWorldInputEvent.DRAG_START,
        modifiers: { altKey: false, ctrlKey: false, metaKey: false, shiftKey: false },
      },
      // The travel before it became a drag included, so the view does not jump.
      { dx: 20, dy: 5, kind: EWorldInputEvent.DRAG_MOVE },
      { kind: EWorldInputEvent.DRAG_END, reason: EWorldDragEndReason.RELEASED },
    ]);
    expect(onClick).not.toHaveBeenCalled();
    expect(element.style.cursor).toBe("");

    target.dispose();
  });

  it("hands on where the pointer rests once a frame, and nothing once it leaves or presses", () => {
    const { element } = mockTree();
    const { inputs, viewport } = mockViewport();
    const onHover = jest.fn();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport, { onHover });

    element.dispatchEvent(pointer("pointermove", { buttons: 0, clientX: 20, clientY: 30 }));
    element.dispatchEvent(pointer("pointermove", { buttons: 0, clientX: 22, clientY: 31 }));
    jest.advanceTimersByTime(100);

    // The latest point of the frame alone, in css pixels from the element's own corner, as the world hears it.
    expect(onHover).toHaveBeenCalledTimes(1);
    expect(onHover).toHaveBeenLastCalledWith({ x: 12, y: 11 });
    expect(inputs).toEqual([{ kind: EWorldInputEvent.HOVER, x: 12, y: 11 }]);

    element.dispatchEvent(pointer("pointerleave", {}));

    expect(onHover).toHaveBeenLastCalledWith(null);

    element.dispatchEvent(pointer("pointermove", { buttons: 0, clientX: 22, clientY: 31 }));
    element.dispatchEvent(pointer("pointerdown", { button: 0, buttons: 1, clientX: 22, clientY: 31 }));
    jest.advanceTimersByTime(100);

    expect(onHover).toHaveBeenLastCalledWith(null);

    target.dispose();
  });

  it("forwards keys once while held", () => {
    const { element } = mockTree();
    const { inputs, viewport } = mockViewport();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport);

    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW" }));
    element.dispatchEvent(new KeyboardEvent("keydown", { code: "KeyW", repeat: true }));
    element.dispatchEvent(new KeyboardEvent("keyup", { code: "KeyW" }));

    expect(inputs).toEqual([
      { code: "KeyW", kind: EWorldInputEvent.KEY_DOWN },
      { code: "KeyW", kind: EWorldInputEvent.KEY_UP },
    ]);

    target.dispose();
  });

  it("keeps Alt to itself, a modifier of the scene's own, while other keys still reach the window", () => {
    const { element } = mockTree();
    const { inputs, viewport } = mockViewport();
    const target: NativeViewportTarget = new NativeViewportTarget(element, viewport);
    const heard: Array<string> = [];

    function onWindowKey(event: KeyboardEvent): void {
      heard.push(event.key);
    }

    window.addEventListener("keydown", onWindowKey);

    element.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, code: "AltLeft", key: "Alt" }));
    element.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, code: "F1", key: "F1" }));

    window.removeEventListener("keydown", onWindowKey);

    expect(inputs).toEqual([
      { code: "AltLeft", kind: EWorldInputEvent.KEY_DOWN },
      { code: "F1", kind: EWorldInputEvent.KEY_DOWN },
    ]);
    expect(heard).toEqual(["F1"]);

    target.dispose();
  });
});
