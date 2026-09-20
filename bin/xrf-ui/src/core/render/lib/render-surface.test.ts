import { describe, expect, it } from "@jest/globals";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import {
  getRenderSurface,
  IRenderSurface,
  isAlphaRenderSurface,
  OPAQUE_RENDER_SURFACE,
  toRenderSurface,
} from "@/core/render/lib/render-surface";
import { mockAlphaSurfaceDescriptor, mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

describe("toRenderSurface", () => {
  it("draws a surface with no answer the way the engine draws an unresolved shader", () => {
    expect(toRenderSurface(null)).toEqual(OPAQUE_RENDER_SURFACE);
    expect(toRenderSurface(mockSurfaceDescriptor({ declaration: { kind: "noLibrary" } }))).toEqual(
      OPAQUE_RENDER_SURFACE
    );
  });

  it("cuts out against the resolved reference without compositing", () => {
    // A cut-out is opaque everywhere it is not discarded, so it keeps writing depth and stays out of the sorted pass.
    const surface: IRenderSurface = toRenderSurface(mockAlphaSurfaceDescriptor());

    expect(surface).toEqual({ alphaTest: 200 / 255, isDepthWritten: true, isTransparent: false });
  });

  it("composites a blended surface and stops it writing depth", () => {
    const surface: IRenderSurface = toRenderSurface(
      mockAlphaSurfaceDescriptor({ draw: { kind: "blended", reference: 32 } })
    );

    expect(surface).toEqual({ alphaTest: 32 / 255, isDepthWritten: false, isTransparent: true });
  });

  it("keeps a blended surface with no reference sampling everything it draws", () => {
    // `models\window` and every other `MODELEbB`: the class passes no reference, so nothing is discarded.
    expect(toRenderSurface(mockAlphaSurfaceDescriptor({ draw: { kind: "blended", reference: 0 } }))).toEqual({
      alphaTest: 0,
      isDepthWritten: false,
      isTransparent: true,
    });
  });
});

describe("isAlphaRenderSurface", () => {
  it("counts a blended surface that discards nothing", () => {
    expect(isAlphaRenderSurface(OPAQUE_RENDER_SURFACE)).toBe(false);
    expect(isAlphaRenderSurface({ alphaTest: 0, isDepthWritten: false, isTransparent: true })).toBe(true);
    expect(isAlphaRenderSurface({ alphaTest: 0.5, isDepthWritten: true, isTransparent: false })).toBe(true);
  });
});

describe("getRenderSurface", () => {
  it("draws a name the table has no answer for the way the engine draws an unresolved shader", () => {
    const surfaces: Record<string, XraySurfaceDescriptor> = { "levels\\aref": mockAlphaSurfaceDescriptor() };

    expect(getRenderSurface(surfaces, "levels\\aref").alphaTest).toBeCloseTo(200 / 255);
    expect(getRenderSurface(surfaces, "levels\\missing")).toEqual(OPAQUE_RENDER_SURFACE);
    expect(getRenderSurface(surfaces, null)).toEqual(OPAQUE_RENDER_SURFACE);
    expect(getRenderSurface({}, "levels\\aref")).toEqual(OPAQUE_RENDER_SURFACE);
  });
});
