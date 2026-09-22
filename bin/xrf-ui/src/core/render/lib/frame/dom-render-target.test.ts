import { describe, expect, it, jest } from "@jest/globals";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { ERenderResolution } from "@/core/render/lib/frame/render-resolution";

function mockContainer(height: number = 540): HTMLElement {
  const container: HTMLElement = document.createElement("div");

  Object.defineProperty(container, "clientHeight", { value: height });
  Object.defineProperty(container, "clientWidth", { value: height * 2 });

  return container;
}

describe("DomRenderTarget", () => {
  it("puts a canvas in the element and takes it away again", () => {
    const container: HTMLElement = mockContainer();
    const target: DomRenderTarget = new DomRenderTarget(container);

    expect(container.querySelector("canvas")).toBe(target.canvas);
    // Filled by css, because on the worker path nothing else can reach its style to size it.
    expect(target.canvas.style.width).toBe("100%");

    target.dispose();

    expect(container.querySelector("canvas")).toBeNull();
  });

  it("draws at what the display is worth until something asks for a height", () => {
    const target: DomRenderTarget = new DomRenderTarget(mockContainer());

    expect(target.pixelRatio).toBe(window.devicePixelRatio);
  });

  // The element stays exactly as it is; how much is drawn into it does not.
  it("draws a chosen height whatever size the element is", () => {
    const target: DomRenderTarget = new DomRenderTarget(mockContainer(540), ERenderResolution.HEIGHT_1080);

    expect(target.pixelRatio).toBe(2);
    expect(target.height).toBe(540);
  });

  it("reports a change of resolution as the resize it is", () => {
    const target: DomRenderTarget = new DomRenderTarget(mockContainer(540));
    const onResized = jest.fn();

    target.observe(onResized);
    target.setResolution(ERenderResolution.HEIGHT_720);

    expect(onResized).toHaveBeenCalledTimes(1);
    expect(target.pixelRatio).toBeCloseTo(720 / 540);

    // Nothing changed, so nothing is redrawn for it.
    target.setResolution(ERenderResolution.HEIGHT_720);

    expect(onResized).toHaveBeenCalledTimes(1);
  });

  it("stops reporting once it is not watched", () => {
    const target: DomRenderTarget = new DomRenderTarget(mockContainer());
    const onResized = jest.fn();

    target.observe(onResized)();
    target.setResolution(ERenderResolution.HEIGHT_2160);

    expect(onResized).not.toHaveBeenCalled();
  });
});
