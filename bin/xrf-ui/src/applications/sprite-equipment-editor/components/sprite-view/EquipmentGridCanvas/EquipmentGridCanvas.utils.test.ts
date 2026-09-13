import { afterEach, describe, expect, it, jest } from "@jest/globals";

import {
  fillRect,
  prepareCanvas,
  strokeRect,
  toHairline,
} from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentGridCanvas/EquipmentGridCanvas.utils";
import { Nullable } from "@/lib/types/general";

/** A context that records what it was asked to do, since jsdom draws nothing. */
function mockContext(): CanvasRenderingContext2D {
  return {
    clearRect: jest.fn(),
    fillRect: jest.fn(),
    setTransform: jest.fn(),
    strokeRect: jest.fn(),
  } as unknown as CanvasRenderingContext2D;
}

/**
 * A canvas element whose context is the recorder above.
 *
 * @param context - Context it hands out, or null to stand for a browser that refused one.
 * @returns The canvas, with its dimensions writable the way a real one is.
 */
function mockCanvas(context: Nullable<CanvasRenderingContext2D>): HTMLCanvasElement {
  return { width: 0, height: 0, getContext: () => context } as unknown as HTMLCanvasElement;
}

describe("prepareCanvas", () => {
  afterEach(() => {
    Reflect.deleteProperty(window, "devicePixelRatio");
  });

  it("sizes the backing store in device pixels and draws in css pixels", () => {
    Object.defineProperty(window, "devicePixelRatio", { configurable: true, value: 2 });

    const context: CanvasRenderingContext2D = mockContext();
    const canvas: HTMLCanvasElement = mockCanvas(context);

    expect(prepareCanvas(canvas, { width: 800, height: 600 })).toBe(context);

    // The buffer is device pixels so a hairline lands on one; everything drawn afterwards is in css pixels.
    expect(canvas.width).toBe(1600);
    expect(canvas.height).toBe(1200);
    expect(context.setTransform).toHaveBeenCalledWith(2, 0, 0, 2, 0, 0);
    expect(context.clearRect).toHaveBeenCalledWith(0, 0, 800, 600);
  });

  it("leaves the backing store alone when the size has not changed", () => {
    Object.defineProperty(window, "devicePixelRatio", { configurable: true, value: 1 });

    const canvas: HTMLCanvasElement = mockCanvas(mockContext());

    prepareCanvas(canvas, { width: 400, height: 300 });

    // Assigning either dimension clears the canvas even when the value is unchanged, which would throw away a frame
    // on every redraw that did not resize.
    const cleared = jest.fn();

    Object.defineProperty(canvas, "width", { configurable: true, get: () => 400, set: cleared });
    prepareCanvas(canvas, { width: 400, height: 300 });

    expect(cleared).not.toHaveBeenCalled();
  });

  it("answers nothing when there is nowhere to draw", () => {
    expect(prepareCanvas(null, { width: 800, height: 600 })).toBeNull();
    expect(prepareCanvas(mockCanvas(mockContext()), { width: 0, height: 600 })).toBeNull();
    expect(prepareCanvas(mockCanvas(mockContext()), { width: 800, height: 0 })).toBeNull();
    expect(prepareCanvas(mockCanvas(null), { width: 800, height: 600 })).toBeNull();
  });
});

describe("toHairline", () => {
  it("puts a one pixel line on a pixel rather than across two", () => {
    expect(toHairline(10)).toBe(10.5);
    expect(toHairline(10.4)).toBe(10.5);
    expect(toHairline(10.6)).toBe(11.5);
  });
});

describe("fillRect and strokeRect", () => {
  it("shades a rectangle exactly as given", () => {
    const context: CanvasRenderingContext2D = mockContext();

    fillRect(context, { x: 10, y: 20, width: 30, height: 40 }, "red");

    expect(context.fillStyle).toBe("red");
    expect(context.fillRect).toHaveBeenCalledWith(10, 20, 30, 40);
  });

  it("insets an outline so it sits inside the bounds it describes", () => {
    const context: CanvasRenderingContext2D = mockContext();

    strokeRect(context, { x: 10, y: 20, width: 30, height: 40 }, "blue", 2);

    expect(context.strokeStyle).toBe("blue");
    expect(context.lineWidth).toBe(2);
    expect(context.strokeRect).toHaveBeenCalledWith(10.5, 20.5, 29, 39);
  });
});
