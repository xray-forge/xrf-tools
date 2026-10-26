import { describe, expect, it } from "@jest/globals";

import {
  ERenderResolution,
  RENDER_OUTPUT_SCALES,
  toRenderOutput,
  toRenderOutputScale,
  toRenderResolution,
} from "@/core/render/lib/settings/render-resolution";

describe("render resolution", () => {
  it("resolves at the viewport's size times the scale while automatic, and at a height asked otherwise", () => {
    expect(toRenderOutput(ERenderResolution.AUTO, 1.25)).toEqual({ outputHeight: null, outputScale: 1.25 });
    expect(toRenderOutput(ERenderResolution.HEIGHT_1440, 1.25)).toEqual({ outputHeight: 1440, outputScale: 1 });
  });

  it("reads a stored scale within the offered range, and the default otherwise", () => {
    expect(toRenderOutputScale("1.5")).toBe(1.5);
    expect(toRenderOutputScale("0.25")).toBe(RENDER_OUTPUT_SCALES.initial);
    expect(toRenderOutputScale("wide")).toBe(RENDER_OUTPUT_SCALES.initial);
    expect(toRenderOutputScale(null)).toBe(RENDER_OUTPUT_SCALES.initial);
  });

  it("reads the old window choice as automatic", () => {
    expect(toRenderResolution("window")).toBe(ERenderResolution.AUTO);
    expect(toRenderResolution("1080")).toBe(ERenderResolution.HEIGHT_1080);
  });
});
