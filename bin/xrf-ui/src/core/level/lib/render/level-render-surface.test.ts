import { describe, expect, it } from "@jest/globals";
import { ERendererDraw, IRendererSurface } from "@xrf/renderer";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import {
  LEVEL_IMPOSTOR_SHADER,
  toLevelSurface,
  toLevelSurfaceColor,
} from "@/core/level/lib/render/level-render-surface";
import { ILevelSurfaceRender, toLevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";
import { mockSectorSurface } from "@/fixtures/mocks/level.mocks";
import { mockMaterialDescriptor, mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

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
    const surface: IRendererSurface = toLevelSurface(mockSectorSurface({ textureName: "water\\row" }), render);

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
      toLevelSurfaceRender(mockWaterDescriptor("water_studen", true))
    );
    const plain: IRendererSurface = toLevelSurface(
      mockSectorSurface(),
      toLevelSurfaceRender(mockWaterDescriptor("water", false))
    );

    expect(studen.water?.anomaly).toEqual({
      isFoamed: true,
      isReflecting: true,
      isSpecular: true,
      isTransparent: false,
    });
    expect(plain.water).toEqual({ anomaly: null, isSoft: false });
  });
});

describe("toLevelSurface for a bumped surface", () => {
  const bumped: XraySurfaceDescriptor = mockSurfaceDescriptor({ bump: mockMaterialDescriptor().bump, material: 2.5 });

  it("binds the pair its base texture declares and shades with the base texture's model", () => {
    const surface: IRendererSurface = toLevelSurface(mockSectorSurface(), toLevelSurfaceRender(bumped));

    expect(surface.textures.bump).toBe("wpn\\wpn_ak74_bump");
    expect(surface.textures.bumpCompanion).toBe("wpn\\wpn_ak74_bump#");
    expect(surface.material).toBe(2.5);
  });

  it("shades a surface the backend described nothing for with the engine's default model", () => {
    expect(toLevelSurfaceRender(null)).toMatchObject({ bump: null, material: 1 });
  });
});

// Textures off is the renderer's setting: a surface never changes with it, so turning them back on fetches nothing.
describe("toLevelSurface's colour", () => {
  it("names every texture the entry binds beside the entry's own colour, which the renderer draws without them", () => {
    const surface: IRendererSurface = toLevelSurface(
      mockSectorSurface({ hemi: "lmap#1", shaderId: 7, textureName: "ston\\wall" }),
      toLevelSurfaceRender(mockSurfaceDescriptor({ detail: { reference: "detail\\detail_stone", scale: 4 } }))
    );

    expect(surface.color).toEqual(toLevelSurfaceColor(7));
    expect(surface.textures).toMatchObject({ base: "ston\\wall", detail: "detail\\detail_stone", hemi: "lmap#1" });
  });

  it("gives water and impostors the entry's colour too, beside their textures", () => {
    const water: IRendererSurface = toLevelSurface(
      mockSectorSurface({ shaderId: 3 }),
      toLevelSurfaceRender(mockWaterDescriptor("water_soft", true))
    );
    const impostor: IRendererSurface = toLevelSurface(
      mockSectorSurface({ shaderId: 4, shaderName: LEVEL_IMPOSTOR_SHADER, textureName: "trees\\lod" }),
      toLevelSurfaceRender(null)
    );

    expect(water.color).toEqual(toLevelSurfaceColor(3));
    expect(water.textures.foam).toBe("water\\water_foam");
    expect(impostor.color).toEqual(toLevelSurfaceColor(4));
    expect(impostor.textures).toEqual({ base: "trees\\lod", hemi: "trees\\lod_nm" });
  });
});
