import { describe, expect, it } from "@jest/globals";
import { ERendererDraw, IRendererSurface } from "@xrf/renderer";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { toLevelSurface } from "@/core/level/lib/render/level-render-surface";
import { ILevelSurfaceRender, toLevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";
import { mockSectorSurface } from "@/fixtures/mocks/level.mocks";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

/** `effects\water` as a script declares it: its program, and what each of its passes binds. */
function mockWaterDescriptor(program: string, isSoft: boolean): XraySurfaceDescriptor {
  return mockSurfaceDescriptor({
    declaration: {
      function: "normal",
      isAlphaTested: false,
      isBlended: isSoft,
      isDepthWritten: false,
      isWallmark: false,
      kind: "scripted",
      program,
      script: "shaders\\r2\\effects_water.s",
    },
    draw: { isSoft, kind: "water" },
    samplers: [
      { element: "normal", name: "s_base", texture: "water\\water_water" },
      { element: "normal", name: "s_nmap", texture: "water\\water_normal" },
      { element: "normal", name: "s_leaves", texture: "water\\water_foam" },
      { element: "l_special", name: "s_base", texture: "water\\water_water" },
      { element: "l_special", name: "s_distort", texture: "water\\water_dudv" },
    ],
  });
}

describe("toLevelSurface for water", () => {
  it("draws water by the textures its script binds, in place of the row's, as OpenXRay's water", () => {
    const render: ILevelSurfaceRender = toLevelSurfaceRender(mockWaterDescriptor("water_soft", true));
    const surface: IRendererSurface = toLevelSurface(mockSectorSurface({ textureName: "water\\row" }), render, true);

    expect(surface.draw).toBe(ERendererDraw.WATER);
    expect(surface.water).toEqual({ anomaly: null, isSoft: true });
    expect(surface.textures).toEqual({
      base: "water\\water_water",
      distortion: "water\\water_dudv",
      foam: "water\\water_foam",
      normal: "water\\water_normal",
    });
  });

  // Anomaly's programs each define their own switches before including its `water.ps`.
  it("draws one of Anomaly's water programs by Anomaly's model, and any other by OpenXRay's", () => {
    const studen: IRendererSurface = toLevelSurface(
      mockSectorSurface(),
      toLevelSurfaceRender(mockWaterDescriptor("water_studen", true)),
      true
    );
    const plain: IRendererSurface = toLevelSurface(
      mockSectorSurface(),
      toLevelSurfaceRender(mockWaterDescriptor("water", false)),
      true
    );

    expect(studen.water?.anomaly).toEqual({
      isFoamed: true,
      isReflecting: true,
      isSpecular: true,
      isTransparent: false,
    });
    expect(plain.water).toEqual({ anomaly: null, isSoft: false });
  });

  it("keeps the colour and the normal map of untextured water, dropping only what it is dressed with", () => {
    const surface: IRendererSurface = toLevelSurface(
      mockSectorSurface(),
      toLevelSurfaceRender(mockWaterDescriptor("water_soft", true)),
      false
    );

    expect(surface.color).toBeDefined();
    expect(surface.textures.base).toBeUndefined();
    expect(surface.textures.normal).toBe("water\\water_normal");
  });
});
