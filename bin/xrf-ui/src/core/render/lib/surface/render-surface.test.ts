import { describe, expect, it } from "@jest/globals";
import { AdditiveBlending, CustomBlending, DstColorFactor, NormalBlending, SrcColorFactor, ZeroFactor } from "three";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import {
  getRenderSurface,
  IRenderSurface,
  isAlphaRenderSurface,
  OPAQUE_RENDER_SURFACE,
  toRenderSurface,
} from "@/core/render/lib/surface/render-surface";
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

    expect(surface).toEqual({ ...OPAQUE_RENDER_SURFACE, alphaTest: 200 / 255 });
  });

  it("composites a blended surface and stops it writing depth", () => {
    const surface: IRenderSurface = toRenderSurface(
      mockAlphaSurfaceDescriptor({ draw: { kind: "blended", reference: 32 } })
    );

    expect(surface).toEqual({
      ...OPAQUE_RENDER_SURFACE,
      alphaTest: 32 / 255,
      isDepthWritten: false,
      isTransparent: true,
    });
    expect(surface.blend.blending).toBe(NormalBlending);
  });

  it("keeps a blended surface with no reference sampling everything it draws", () => {
    // `models\window` and every other `MODELEbB`: the class passes no reference, so nothing is discarded.
    expect(toRenderSurface(mockAlphaSurfaceDescriptor({ draw: { kind: "blended", reference: 0 } }))).toEqual({
      ...OPAQUE_RENDER_SURFACE,
      isDepthWritten: false,
      isTransparent: true,
    });
  });

  // `effects\glow`, which a level names for every lamp: added rather than composited, so it brightens what is
  // behind it. Drawn opaque it was a black rectangle standing in the air.
  it("adds a glow to what is behind it", () => {
    const surface: IRenderSurface = toRenderSurface(
      mockAlphaSurfaceDescriptor({ draw: { kind: "added", reference: 255 } })
    );

    expect(surface.blend.blending).toBe(AdditiveBlending);
    expect(surface.alphaTest).toBeCloseTo(1);
    expect(surface.isDepthWritten).toBe(false);
    expect(surface.isTransparent).toBe(true);
  });

  // `effects\wallmarkmult`, which a level names for every decal. Three.js has no named blending for either
  // multiply, so both are stated as factors.
  it("multiplies a decal into the surface it is laid on", () => {
    const single: IRenderSurface = toRenderSurface(
      mockAlphaSurfaceDescriptor({ draw: { isDoubled: false, kind: "multiplied" } })
    );
    const doubled: IRenderSurface = toRenderSurface(
      mockAlphaSurfaceDescriptor({ draw: { isDoubled: true, kind: "multiplied" } })
    );

    expect(single.blend).toEqual({ blendDst: ZeroFactor, blendSrc: DstColorFactor, blending: CustomBlending });
    expect(doubled.blend).toEqual({ blendDst: SrcColorFactor, blendSrc: DstColorFactor, blending: CustomBlending });
    // The multiplying equations state no reference, so nothing is discarded before the multiply.
    expect(doubled.alphaTest).toBe(0);
  });

  it("carries the detail texture an opaque surface is modulated with", () => {
    const surface: IRenderSurface = toRenderSurface(
      mockSurfaceDescriptor({ detail: { reference: "detail\\detail_grnd_earth", scale: 150 } })
    );

    expect(surface).toEqual({
      ...OPAQUE_RENDER_SURFACE,
      detail: { reference: "detail\\detail_grnd_earth", scale: 150 },
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
