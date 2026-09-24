import { describe, expect, it } from "@jest/globals";

import {
  DEFAULT_RENDER_RESOLUTION,
  ERenderResolution,
  toRenderPixelRatio,
  toRenderResolution,
} from "#/frame/render-resolution";

describe("toRenderResolution", () => {
  it("takes one of the offered choices", () => {
    expect(toRenderResolution("1080")).toBe(ERenderResolution.HEIGHT_1080);
    expect(toRenderResolution("1440")).toBe(ERenderResolution.HEIGHT_1440);
    expect(toRenderResolution("window")).toBe(ERenderResolution.WINDOW);
  });

  // Written by an older build, a hand edit, or a half-finished rename: falling back beats drawing at nothing.
  it("refuses anything else", () => {
    expect(toRenderResolution("8k")).toBe(DEFAULT_RENDER_RESOLUTION);
    expect(toRenderResolution(null)).toBe(DEFAULT_RENDER_RESOLUTION);
    expect(toRenderResolution(1080)).toBe(DEFAULT_RENDER_RESOLUTION);
  });
});

describe("toRenderPixelRatio", () => {
  it("draws at what the display is worth when the window is the answer", () => {
    expect(toRenderPixelRatio(ERenderResolution.WINDOW, 1000, 2)).toBe(2);
  });

  // A chosen height is exactly that height, whatever the element is: half the ratio on a viewport twice as tall.
  it("draws a chosen height at whatever ratio reaches it", () => {
    expect(toRenderPixelRatio(ERenderResolution.HEIGHT_1080, 1080, 1)).toBe(1);
    expect(toRenderPixelRatio(ERenderResolution.HEIGHT_1080, 540, 1)).toBe(2);
    expect(toRenderPixelRatio(ERenderResolution.HEIGHT_720, 1440, 3)).toBe(0.5);
    expect(toRenderPixelRatio(ERenderResolution.HEIGHT_2160, 1080, 1)).toBe(2);
  });

  // A ratio of zero draws nothing at all, and an unmeasured element is a frame away from being measured.
  it("falls back to the display until the element has a size", () => {
    expect(toRenderPixelRatio(ERenderResolution.HEIGHT_1080, 0, 1.5)).toBe(1.5);
    expect(toRenderPixelRatio(ERenderResolution.HEIGHT_1080, -1, 1.5)).toBe(1.5);
  });
});
