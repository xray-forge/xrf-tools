import { describe, expect, it } from "@jest/globals";
import { ERendererDraw } from "@xrf/renderer";

import { XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw } from "@/core/ipc/types/xrf-material";
import {
  getRendererSurfaceDraw,
  IRendererSurfaceDraw,
  isAlphaRendererSurfaceDraw,
  isLitSurface,
  isWallmarkSurface,
  OPAQUE_RENDERER_SURFACE_DRAW,
  toRendererSurfaceDraw,
} from "@/core/render/lib/surface/renderer-surface-draw";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

function mockScripted(program: string, overrides: Partial<Record<string, boolean>> = {}): XraySurfaceDeclaration {
  return {
    function: "normal",
    isAlphaTested: false,
    isBlended: false,
    isDepthWritten: true,
    isWallmark: false,
    kind: "scripted",
    program,
    script: `shaders\\r2\\${program}.s`,
    ...overrides,
  };
}

function drawOf(draw: XraySurfaceDraw): IRendererSurfaceDraw {
  return toRendererSurfaceDraw(mockSurfaceDescriptor({ draw }));
}

describe("toRendererSurfaceDraw", () => {
  it("draws a surface nobody resolved opaque and lit, as the plain base shader is", () => {
    expect(toRendererSurfaceDraw(null)).toEqual(OPAQUE_RENDERER_SURFACE_DRAW);
  });

  it("maps each draw the backend resolved onto the renderer's, with the reference out of 255", () => {
    expect(drawOf({ kind: "opaque" })).toEqual({ draw: ERendererDraw.OPAQUE, isLit: true });
    expect(drawOf({ kind: "alphaTested", reference: 51 })).toEqual({
      alphaReference: 0.2,
      draw: ERendererDraw.CUT_OUT,
      isLit: true,
    });
    expect(drawOf({ kind: "blended", reference: 0 }).draw).toBe(ERendererDraw.BLENDED);
    expect(drawOf({ isWeighted: false, kind: "added", reference: 0 }).draw).toBe(ERendererDraw.ADDED);
    expect(drawOf({ isWeighted: true, kind: "added", reference: 0 }).draw).toBe(ERendererDraw.ALPHA_ADDED);
    expect(drawOf({ isDoubled: false, kind: "multiplied" }).draw).toBe(ERendererDraw.MULTIPLIED);
    expect(drawOf({ isDoubled: true, kind: "multiplied" }).draw).toBe(ERendererDraw.MULTIPLIED_2X);
    expect(drawOf({ kind: "invisible" }).draw).toBe(ERendererDraw.INVISIBLE);
  });

  // Anomaly's water programs define their switches before including `water.ps`; any other program is OpenXRay's.
  it("draws water by Anomaly's model for one of its programs, and OpenXRay's for any other", () => {
    function water(program: string): IRendererSurfaceDraw {
      return toRendererSurfaceDraw(
        mockSurfaceDescriptor({ declaration: mockScripted(program), draw: { isSoft: true, kind: "water" } })
      );
    }

    expect(water("water_studen")).toEqual({
      draw: ERendererDraw.WATER,
      isLit: true,
      water: {
        anomaly: { isFoamed: true, isReflecting: true, isSpecular: true, isTransparent: false },
        isSoft: true,
      },
    });
    expect(water("water_soft").water?.anomaly).toBeNull();
  });

  it("leaves a scripted blended pass unlit, and lights water whatever its blend says", () => {
    const blended: XraySurfaceDeclaration = mockScripted("wmark", { isBlended: true });

    expect(toRendererSurfaceDraw(mockSurfaceDescriptor({ declaration: blended })).isLit).toBe(false);
    expect(
      toRendererSurfaceDraw(mockSurfaceDescriptor({ declaration: blended, draw: { isSoft: false, kind: "water" } }))
        .isLit
    ).toBe(true);
  });
});

describe("getRendererSurfaceDraw", () => {
  it("joins a surface by its position, opaque past the end of the table", () => {
    const surfaces: Array<XraySurfaceDescriptor> = [
      mockSurfaceDescriptor(),
      mockSurfaceDescriptor({ draw: { kind: "alphaTested", reference: 200 } }),
    ];

    expect(getRendererSurfaceDraw(surfaces, 1).draw).toBe(ERendererDraw.CUT_OUT);
    expect(getRendererSurfaceDraw(surfaces, 7)).toEqual(OPAQUE_RENDERER_SURFACE_DRAW);
  });
});

describe("isAlphaRendererSurfaceDraw", () => {
  // The visuals `Alpha` toggle is offered only where it changes something, which is where a draw reads the channel.
  it("answers yes only for draws that read the base texture's alpha", () => {
    const reading: Array<ERendererDraw> = Object.values(ERendererDraw).filter((draw: ERendererDraw) =>
      isAlphaRendererSurfaceDraw({ draw, isLit: true })
    );

    expect(reading).toEqual([ERendererDraw.CUT_OUT, ERendererDraw.BLENDED, ERendererDraw.ALPHA_ADDED]);
  });
});

describe("isLitSurface", () => {
  it("lights everything but a scripted blended pass", () => {
    expect(isLitSurface(null)).toBe(true);
    expect(isLitSurface(mockSurfaceDescriptor())).toBe(true);
    expect(isLitSurface(mockSurfaceDescriptor({ declaration: mockScripted("wmark", { isBlended: true }) }))).toBe(
      false
    );
  });
});

describe("isWallmarkSurface", () => {
  it("takes a surface for a wall mark only where its script says so", () => {
    expect(isWallmarkSurface(null)).toBe(false);
    expect(isWallmarkSurface(mockSurfaceDescriptor({ declaration: mockScripted("wmark") }))).toBe(false);
    expect(isWallmarkSurface(mockSurfaceDescriptor({ declaration: mockScripted("wmark", { isWallmark: true }) }))).toBe(
      true
    );
  });
});
