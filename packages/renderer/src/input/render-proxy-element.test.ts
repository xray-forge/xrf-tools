import { describe, expect, it, jest } from "@jest/globals";

import { ERenderInput, IRenderInputEvent } from "#/input/render-input";
import { IRenderProxyEvent, RenderProxyElement } from "#/input/render-proxy-element";

const SIZE = { height: 540, pixelRatio: 1, width: 960 };

function mockEvent(type: ERenderInput, overrides: Partial<IRenderInputEvent> = {}): IRenderInputEvent {
  return {
    altKey: false,
    button: 0,
    buttons: 1,
    clientX: 0,
    clientY: 0,
    code: "",
    ctrlKey: false,
    deltaMode: 0,
    deltaX: 0,
    deltaY: 0,
    isPrimary: true,
    metaKey: false,
    pointerId: 1,
    pointerType: "mouse",
    shiftKey: false,
    type,
    ...overrides,
  };
}

describe("RenderProxyElement", () => {
  it("hands a gesture to whatever listens for that one", () => {
    const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());
    const down = jest.fn();
    const up = jest.fn();

    element.addEventListener(ERenderInput.POINTER_DOWN, down);
    element.addEventListener(ERenderInput.POINTER_UP, up);
    element.dispatch(mockEvent(ERenderInput.POINTER_DOWN, { clientX: 12 }));

    expect(down).toHaveBeenCalledTimes(1);
    expect(up).not.toHaveBeenCalled();
    expect((down.mock.calls[0][0] as IRenderProxyEvent).clientX).toBe(12);
  });

  // Three's controls call both on every event they take, and a page is what actually refuses them.
  it("answers the two calls a control makes on an event", () => {
    const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());

    element.addEventListener(ERenderInput.WHEEL, (event: IRenderProxyEvent) => {
      event.preventDefault();
      event.stopPropagation();
    });

    expect(() => element.dispatch(mockEvent(ERenderInput.WHEEL))).not.toThrow();
  });

  // Letting go of a pointer is what a control does from inside the call telling it the pointer went up.
  it("survives a listener that stops listening while it is being told", () => {
    const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());
    const second = jest.fn();

    const first = jest.fn(() => {
      element.removeEventListener(ERenderInput.POINTER_UP, first);
      element.removeEventListener(ERenderInput.POINTER_UP, second);
    });

    element.addEventListener(ERenderInput.POINTER_UP, first);
    element.addEventListener(ERenderInput.POINTER_UP, second);
    element.dispatch(mockEvent(ERenderInput.POINTER_UP));

    expect(first).toHaveBeenCalledTimes(1);
    expect(second).toHaveBeenCalledTimes(1);

    element.dispatch(mockEvent(ERenderInput.POINTER_UP));

    expect(first).toHaveBeenCalledTimes(1);
  });

  it("stops telling a listener that was taken off", () => {
    const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());
    const listener = jest.fn();

    element.addEventListener(ERenderInput.POINTER_MOVE, listener);
    element.removeEventListener(ERenderInput.POINTER_MOVE, listener);
    element.dispatch(mockEvent(ERenderInput.POINTER_MOVE));

    expect(listener).not.toHaveBeenCalled();
  });

  // Three listens to the element for some gestures and to its document for others, and on a thread with
  // neither there is only the one thing to listen to.
  it("is its own document and its own root", () => {
    const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());

    expect(element.ownerDocument).toBe(element);
    expect(element.getRootNode()).toBe(element);
  });

  it("measures itself at the origin, because nothing else is there", () => {
    const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());

    expect(element.clientWidth).toBe(960);
    expect(element.clientHeight).toBe(540);
    expect(element.getBoundingClientRect()).toMatchObject({ height: 540, left: 0, top: 0, width: 960 });

    element.resize({ height: 200, pixelRatio: 2, width: 400 });

    expect(element.getBoundingClientRect()).toMatchObject({ height: 200, width: 400 });
  });

  // Three writes `grab`, `grabbing` and `auto` on the element itself, so the cursor comes back for free.
  it("hands back the cursor the controls asked for", () => {
    const onCursor = jest.fn();
    const element: RenderProxyElement = new RenderProxyElement(SIZE, onCursor);

    element.style.cursor = "grabbing";

    expect(onCursor).toHaveBeenCalledWith("grabbing");
    expect(element.style.cursor).toBe("grabbing");
  });

  it("takes a captured pointer without owning one", () => {
    const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());

    expect(() => {
      element.setPointerCapture();
      element.releasePointerCapture();
    }).not.toThrow();

    expect(element.hasPointerCapture()).toBe(false);
  });
});
