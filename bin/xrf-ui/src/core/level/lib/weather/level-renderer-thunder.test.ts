import { describe, expect, it } from "@jest/globals";
import { ERendererDraw } from "@xrf/renderer";

import { LevelThunderbolts } from "@/core/ipc/types/xrf-app";
import { EXraySurfaceDraw } from "@/core/ipc/types/xrf-material";
import { listLevelThunderTextures, toLevelRendererThunder } from "@/core/level/lib/weather/level-renderer-thunder";

const GRADIENT = {
  logicalPath: "textures\\fx\\fx_thunderbolts_gradient.dds",
  reference: "fx\\fx_thunderbolts_gradient",
};
const LIGHTNING = { logicalPath: "textures\\fx\\fx_lightning.dds", reference: "fx\\fx_lightning" };

/** Vanilla's `collection_default` with one of its bolts, as the backend describes it. */
const THUNDERBOLTS: LevelThunderbolts = {
  animators: [
    {
      fps: 30,
      frameCount: 2,
      keys: [
        { color: [255, 255, 255], frame: 0 },
        { color: [0, 0, 0], frame: 1 },
      ],
    },
  ],
  bolts: [
    {
      center: {
        draw: { isWeighted: true, kind: EXraySurfaceDraw.ADDED, reference: 0 },
        opacity: 0.6,
        radius: [2, 1],
        texture: GRADIENT,
      },
      color: 0,
      model: 0,
      name: "default_00",
      top: { draw: { kind: EXraySurfaceDraw.OPAQUE }, opacity: 0.6, radius: [0.5, 0.25], texture: GRADIENT },
    },
  ],
  collections: [
    { file: "environment\\thunderbolt_collections.ltx", name: "collection_default", thunderbolts: ["default_00"] },
  ],
  models: [
    {
      draw: { isWeighted: false, kind: EXraySurfaceDraw.ADDED, reference: 0 },
      mesh: {
        indices: [0, 1, 2],
        positions: [0, 0, 0, 0, -1, 0, 0.1, null, 0],
        texture: LIGHTNING,
        uvs: [0, 0, 1, 0, 0, 1],
      },
      name: "dm\\dm_lightning-01.dm",
    },
  ],
  settings: {
    altitude: [0.35, 0.35],
    deltaLongitude: 0.52,
    fogColor: 0.1,
    minDistance: 0.94,
    secondProbability: 0.5,
    skyColor: 0.1,
    sunColor: 0.9,
    tilt: 0.3,
  },
};

describe("toLevelRendererThunder", () => {
  it("hands the renderer the collections, bolts, models and animators by name and reference", () => {
    const thunder = toLevelRendererThunder(THUNDERBOLTS);

    expect(thunder?.collections).toEqual({ collection_default: ["default_00"] });
    expect(thunder?.bolts.default_00).toEqual({
      center: {
        draw: ERendererDraw.ALPHA_ADDED,
        opacity: 0.6,
        radius: [2, 1],
        texture: "fx\\fx_thunderbolts_gradient",
      },
      color: 0,
      model: 0,
      // A shader that composites nothing still lights the air.
      top: { draw: ERendererDraw.ADDED, opacity: 0.6, radius: [0.5, 0.25], texture: "fx\\fx_thunderbolts_gradient" },
    });
    expect(thunder?.models[0]).toMatchObject({ draw: ERendererDraw.ADDED, positions: [0, 0, 0, 0, -1, 0, 0.1, 0, 0] });
    expect(thunder?.animators[0]).toEqual({
      colors: [
        [255, 255, 255],
        [0, 0, 0],
      ],
      fps: 30,
      frameCount: 2,
      frames: [0, 1],
    });
    expect(thunder?.settings?.sunColor).toBe(0.9);
  });

  it("strikes nothing for a game that says nowhere bolts strike", () => {
    expect(toLevelRendererThunder({ ...THUNDERBOLTS, settings: null })).toBeNull();
  });

  it("lists the textures the models and glows draw with", () => {
    expect(listLevelThunderTextures(THUNDERBOLTS)).toEqual([LIGHTNING, GRADIENT, GRADIENT]);
  });
});
