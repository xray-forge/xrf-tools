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

    expect(surface).toEqual({ alphaTest: 200 / 255, detail: null, isDepthWritten: true, isTransparent: false });
  });

  it("composites a blended surface and stops it writing depth", () => {
    const surface: IRenderSurface = toRenderSurface(
      mockAlphaSurfaceDescriptor({ draw: { kind: "blended", reference: 32 } })
    );

    expect(surface).toEqual({ alphaTest: 32 / 255, detail: null, isDepthWritten: false, isTransparent: true });
  });

  it("keeps a blended surface with no reference sampling everything it draws", () => {
    // `models\window` and every other `MODELEbB`: the class passes no reference, so nothing is discarded.
    expect(toRenderSurface(mockAlphaSurfaceDescriptor({ draw: { kind: "blended", reference: 0 } }))).toEqual({
      alphaTest: 0,
      detail: null,
      isDepthWritten: false,
      isTransparent: true,
    });
  });

  it("carries the detail texture an opaque surface is modulated with", () => {
    const surface: IRenderSurface = toRenderSurface(
      mockSurfaceDescriptor({ detail: { reference: "detail\\detail_grnd_earth", scale: 150 } })
    );

    expect(surface).toEqual({
      alphaTest: 0,
      detail: { reference: "detail\\detail_grnd_earth", scale: 150 },
      isDepthWritten: true,
      isTransparent: false,
    });
  });

  // A tiling is what lays the detail out, and the wire carries none for a value the engine never bound a scaler for.
  // Modulating at some invented density would be worse than not modulating.
  it("drops a detail texture that arrives without a tiling", () => {
    const surface: IRenderSurface = toRenderSurface(
      mockSurfaceDescriptor({ detail: { reference: "detail\\detail_grnd_earth", scale: null } })
    );

    expect(surface.detail).toBeNull();
  });
});

describe("isAlphaRenderSurface", () => {
  it("counts a blended surface that discards nothing", () => {
    expect(isAlphaRenderSurface(OPAQUE_RENDER_SURFACE)).toBe(false);
    expect(isAlphaRenderSurface({ ...OPAQUE_RENDER_SURFACE, isDepthWritten: false, isTransparent: true })).toBe(true);
    expect(isAlphaRenderSurface({ ...OPAQUE_RENDER_SURFACE, alphaTest: 0.5 })).toBe(true);
  });
});

describe("getRenderSurface", () => {
  // Addressed by the position of whatever declared the surface, so a table row the backend answered nothing for and a
  // row past the end of the table both draw the way the engine draws an unresolved shader.
  it("draws an index the table does not reach the way the engine draws an unresolved shader", () => {
    const surfaces: Array<XraySurfaceDescriptor> = [mockSurfaceDescriptor(), mockAlphaSurfaceDescriptor()];

    expect(getRenderSurface(surfaces, 1).alphaTest).toBeCloseTo(200 / 255);
    expect(getRenderSurface(surfaces, 0)).toEqual(OPAQUE_RENDER_SURFACE);
    expect(getRenderSurface(surfaces, 7)).toEqual(OPAQUE_RENDER_SURFACE);
    expect(getRenderSurface([], 0)).toEqual(OPAQUE_RENDER_SURFACE);
  });

  // Two rows over one shader, told apart only by the textures they dress with, which a name-keyed lookup could not do.
  it("answers two rows naming one shader with their own detail", () => {
    const surfaces: Array<XraySurfaceDescriptor> = [
      mockSurfaceDescriptor({ detail: { reference: "detail\\detail_dirt_det1", scale: 8 } }),
      mockSurfaceDescriptor({ detail: { reference: "detail\\detail_grnd_grass", scale: 4 } }),
    ];

    expect(getRenderSurface(surfaces, 0).detail?.reference).toBe("detail\\detail_dirt_det1");
    expect(getRenderSurface(surfaces, 1).detail?.reference).toBe("detail\\detail_grnd_grass");
  });
});
