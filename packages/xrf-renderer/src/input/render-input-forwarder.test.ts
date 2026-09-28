/**
 * @jest-environment jsdom
 */

import { describe, expect, it } from "@jest/globals";

import { ERenderInput } from "#/contract/render-input";
import { IRenderInputEvent } from "#/contract/render-input-event";
import { RenderInputForwarder } from "#/input/render-input-forwarder";

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
  it("keeps sending a drag that has left the canvas, until it is let go", () => {
    const { canvas, sent, forwarder } = mockForwarder();

    sendTo(canvas, ERenderInput.POINTER_DOWN, { pointerId: 1 });
    sendTo(window, ERenderInput.POINTER_MOVE, { clientX: 99, pointerId: 1 });
    sendTo(window, ERenderInput.POINTER_UP, { pointerId: 1 });
    sendTo(window, ERenderInput.POINTER_MOVE, { clientX: 120, pointerId: 1 });

    expect(sent.map((it) => it.type)).toEqual([
      ERenderInput.POINTER_DOWN,
      ERenderInput.POINTER_MOVE,
      ERenderInput.POINTER_UP,
    ]);

    forwarder.dispose();
  });

  it("sends nothing of a pointer only passing over the window", () => {
    const { sent, forwarder } = mockForwarder();

    sendTo(window, ERenderInput.POINTER_MOVE, { clientX: 99 });

    expect(sent).toEqual([]);

    forwarder.dispose();
  });

  it("sends nothing of another pointer moving while one pressed on the canvas is down", () => {
    const { canvas, sent, forwarder } = mockForwarder();

    sendTo(canvas, ERenderInput.POINTER_DOWN, { pointerId: 1 });
    sendTo(window, ERenderInput.POINTER_MOVE, { pointerId: 2 });
    sendTo(window, ERenderInput.POINTER_UP, { pointerId: 2 });
    sendTo(window, ERenderInput.POINTER_MOVE, { pointerId: 1 });

    expect(sent.map((it: IRenderInputEvent) => [it.type, it.pointerId])).toEqual([
      [ERenderInput.POINTER_DOWN, 1],
      [ERenderInput.POINTER_MOVE, 1],
    ]);

    forwarder.dispose();
  });

  // Alt-tab mid drag: the let go happens in another window, and the drag would otherwise go on once back.
  it("cancels every drag still down once the window loses focus, and stops listening to it", () => {
    const { canvas, sent, forwarder } = mockForwarder();

    sendTo(canvas, ERenderInput.POINTER_DOWN, { pointerId: 4 });
    window.dispatchEvent(new Event("blur"));
    sendTo(window, ERenderInput.POINTER_MOVE, { pointerId: 4 });

    expect(sent.map((it: IRenderInputEvent) => [it.type, it.pointerId])).toEqual([
      [ERenderInput.POINTER_DOWN, 4],
      [ERenderInput.POINTER_CANCEL, 4],
    ]);

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

  // A camera flown by the keys hears them only once a press has focused the canvas.
  it("focuses the canvas on a press and sends its keys by where they sit", () => {
    const { canvas, sent, forwarder } = mockForwarder();

    document.body.appendChild(canvas);
    sendTo(canvas, ERenderInput.POINTER_DOWN);

    expect(document.activeElement).toBe(canvas);

    sendTo(canvas, ERenderInput.KEY_DOWN, { code: "KeyW" });
    sendTo(canvas, ERenderInput.KEY_UP, { code: "KeyW" });
    sendTo(canvas, ERenderInput.BLUR);

    expect(sent.slice(1).map((it) => [it.type, it.code])).toEqual([
      [ERenderInput.KEY_DOWN, "KeyW"],
      [ERenderInput.KEY_UP, "KeyW"],
      [ERenderInput.BLUR, ""],
    ]);

    forwarder.dispose();
    canvas.remove();
  });

  // Tab and every shortcut keep working; only the keys that would scroll the page under the canvas are refused.
  it("refuses only the keys that scroll", () => {
    const { canvas, forwarder } = mockForwarder();

    expect(sendTo(canvas, ERenderInput.KEY_DOWN, { code: "ArrowUp" }).defaultPrevented).toBe(true);
    expect(sendTo(canvas, ERenderInput.KEY_DOWN, { code: "KeyW" }).defaultPrevented).toBe(false);
    expect(sendTo(canvas, ERenderInput.KEY_DOWN, { code: "Tab" }).defaultPrevented).toBe(false);

    forwarder.dispose();
  });

  it("gives the canvas its focus order back", () => {
    const { canvas, forwarder } = mockForwarder();

    expect(canvas.tabIndex).toBe(0);

    forwarder.dispose();

    expect(canvas.tabIndex).toBe(-1);
  });

  // The far side's controls outlive the canvas: told nothing, they would hold the pointer into the next view.
  it("cancels every pointer still down as it is disposed", () => {
    const { canvas, sent, forwarder } = mockForwarder();

    sendTo(canvas, ERenderInput.POINTER_DOWN, { pointerId: 4 });
    sent.length = 0;
    forwarder.dispose();

    expect(sent).toEqual([expect.objectContaining({ pointerId: 4, type: ERenderInput.POINTER_CANCEL })]);
  });

  it("stops sending once it is disposed", () => {
    const { canvas, sent, forwarder } = mockForwarder();

    sendTo(canvas, ERenderInput.POINTER_DOWN);
    forwarder.dispose();
    sent.length = 0;

    sendTo(canvas, ERenderInput.POINTER_DOWN);
    sendTo(window, ERenderInput.POINTER_MOVE);

    expect(sent).toEqual([]);
  });
});
