import { describe, expect, it } from "@jest/globals";

import { IRendererViewSize } from "#/contract/renderer-view-size";
import { RendererView } from "#/host/renderer-view";

/**
 * @param size - What the page measured.
 * @returns The device pixels three draws for the size the view takes: css times the ratio, floored.
 */
function toDrawn(size: IRendererViewSize): readonly [number, number] {
  const { width, height, pixelRatio } = new RendererView({} as OffscreenCanvas, size).size;

  return [Math.floor(width * pixelRatio), Math.floor(height * pixelRatio)];
}

describe("RendererView", () => {
  it("draws what the page measured", () => {
    expect(toDrawn({ height: 480, pixelRatio: 1.5, width: 640 })).toEqual([960, 720]);
  });

  // Nothing allocates an empty target: a hidden element measures nothing.
  it("draws a pixel of an element that measures nothing", () => {
    expect(toDrawn({ height: 0, pixelRatio: 1, width: 0 })).toEqual([1, 1]);
  });

  // A fixed drawing height over a tall element gives a ratio under one, which floors a css pixel to none.
  it("draws a device pixel at least of a side whose css pixels a ratio under one floors to none", () => {
    for (const pixelRatio of [0.72, 0.5, 1 / 3, 0.0999]) {
      const [width, height] = toDrawn({ height: 1, pixelRatio, width: 0 });

      expect(width).toBeGreaterThanOrEqual(1);
      expect(height).toBeGreaterThanOrEqual(1);
    }

    expect(toDrawn({ height: 1000, pixelRatio: 0.72, width: 1 })).toEqual([1, 720]);
  });

  it("takes a resize once", () => {
    const view: RendererView = new RendererView({} as OffscreenCanvas, { height: 1, pixelRatio: 1, width: 1 });

    expect(view.takeResize()).toBe(true);
    expect(view.takeResize()).toBe(false);

    view.resize({ height: 2, pixelRatio: 1, width: 2 });

    expect(view.takeResize()).toBe(true);
    expect(view.size).toEqual({ height: 2, pixelRatio: 1, width: 2 });
  });
});
