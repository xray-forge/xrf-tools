import { describe, expect, it } from "@jest/globals";
import {
  DstColorFactor,
  OneFactor,
  OneMinusSrcAlphaFactor,
  SrcAlphaFactor,
  SrcColorFactor,
  ZeroFactor,
} from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-surface";
import { toSurfaceCompositing } from "#/material/surface-compositing";

describe("toSurfaceCompositing", () => {
  it("leaves what the G-buffer takes uncomposited", () => {
    expect(toSurfaceCompositing({ draw: ERendererDraw.OPAQUE })).toBeNull();
    expect(toSurfaceCompositing({ draw: ERendererDraw.CUT_OUT, isWallmark: true })).toBeNull();
  });

  it("blends each draw by the engine's factors, keeping the alpha under all but a blended one", () => {
    expect(toSurfaceCompositing({ draw: ERendererDraw.BLENDED })).toMatchObject({
      alphaDestination: OneMinusSrcAlphaFactor,
      alphaSource: OneFactor,
      destination: OneMinusSrcAlphaFactor,
      source: SrcAlphaFactor,
    });
    expect(toSurfaceCompositing({ draw: ERendererDraw.ADDED })).toMatchObject({
      alphaDestination: OneFactor,
      alphaSource: ZeroFactor,
      destination: OneFactor,
      source: OneFactor,
    });
    expect(toSurfaceCompositing({ draw: ERendererDraw.ALPHA_ADDED })).toMatchObject({
      destination: OneFactor,
      source: SrcAlphaFactor,
    });
    expect(toSurfaceCompositing({ draw: ERendererDraw.MULTIPLIED })).toMatchObject({
      destination: ZeroFactor,
      source: DstColorFactor,
    });
    expect(toSurfaceCompositing({ draw: ERendererDraw.MULTIPLIED_2X })).toMatchObject({
      destination: SrcColorFactor,
      source: DstColorFactor,
    });
  });

  it("writes no colour for an invisible surface", () => {
    expect(toSurfaceCompositing({ draw: ERendererDraw.INVISIBLE })?.isColorWritten).toBe(false);
    expect(toSurfaceCompositing({ draw: ERendererDraw.ADDED })?.isColorWritten).toBe(true);
  });

  // `dx10color_write_enable(true, true, true, false)`: the gloss in the albedo's alpha survives every wall mark.
  it("keeps the alpha under a wall mark whatever its draw", () => {
    expect(toSurfaceCompositing({ draw: ERendererDraw.BLENDED, isWallmark: true })).toEqual({
      alphaDestination: OneFactor,
      alphaSource: ZeroFactor,
      destination: OneMinusSrcAlphaFactor,
      isColorWritten: true,
      source: SrcAlphaFactor,
    });
  });
});
