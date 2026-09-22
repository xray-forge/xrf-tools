import { describe, expect, it } from "@jest/globals";

import { ERenderInput, IRenderInputEvent } from "@/core/render/lib/worker/render-input";
import { RenderInputForwarder } from "@/core/render/lib/worker/render-input-forwarder";

function sendTo(target: EventTarget, type: ERenderInput, fields: Record<string, unknown> = {}): Event {
  const event: Event = Object.assign(new Event(type, { bubbles: true, cancelable: true }), fields);

  target.dispatchEvent(event);

  return event;
}

function mockForwarder(): {
  canvas: HTMLCanvasElement;
  sent: Array<IRenderInputEvent>;
  forwarder: RenderInputForwarder;
} {
  const canvas: HTMLCanvasElement = document.createElement("canvas");
  const sent: Array<IRenderInputEvent> = [];

  return { canvas, forwarder: new RenderInputForwarder(canvas, (event) => sent.push(event)), sent };
}

describe("RenderInputForwarder", () => {
  it("sends what was done to the canvas", () => {
    const { canvas, sent, forwarder } = mockForwarder();

    sendTo(canvas, ERenderInput.POINTER_DOWN, { clientX: 20, clientY: 8, pointerId: 3 });

    expect(sent).toHaveLength(1);
    expect(sent[0]).toMatchObject({ clientX: 20, clientY: 8, pointerId: 3, type: ERenderInput.POINTER_DOWN });

    forwarder.dispose();
  });

  // Pointer capture cannot help a thread that does not receive the events, so the window is what is watched.
  it("keeps sending a drag that has left the canvas", () => {
    const { sent, forwarder } = mockForwarder();

    sendTo(window, ERenderInput.POINTER_MOVE, { clientX: 99 });
    sendTo(window, ERenderInput.POINTER_UP, {});

    expect(sent.map((it) => it.type)).toEqual([ERenderInput.POINTER_MOVE, ERenderInput.POINTER_UP]);

    forwarder.dispose();
  });

  // A frame spent asking the other thread whether to scroll is a frame the page has already scrolled.
  it("refuses scrolling and the browser's own menu here, where refusing still counts", () => {
    const { canvas, forwarder } = mockForwarder();

    expect(sendTo(canvas, ERenderInput.WHEEL, { deltaY: 120 }).defaultPrevented).toBe(true);
    expect(sendTo(canvas, ERenderInput.CONTEXT_MENU).defaultPrevented).toBe(true);
    expect(canvas.style.touchAction).toBe("none");

    forwarder.dispose();

    expect(canvas.style.touchAction).toBe("");
  });

  it("shows the cursor the far side asked for, and gives it back", () => {
    const { canvas, forwarder } = mockForwarder();

    forwarder.setCursor("grabbing");

    expect(canvas.style.cursor).toBe("grabbing");

    forwarder.dispose();

    expect(canvas.style.cursor).toBe("");
  });

  it("stops sending once it is disposed", () => {
    const { canvas, sent, forwarder } = mockForwarder();

    forwarder.dispose();

    sendTo(canvas, ERenderInput.POINTER_DOWN);
    sendTo(window, ERenderInput.POINTER_MOVE);

    expect(sent).toEqual([]);
  });
});
