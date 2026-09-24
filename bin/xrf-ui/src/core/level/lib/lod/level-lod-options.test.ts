import { describe, expect, it } from "@jest/globals";
import { DEFAULT_RENDERER_LOD_SETTINGS } from "@xrf/renderer";

import { DEFAULT_LEVEL_LOD_OPTIONS, toLevelRendererLod } from "@/core/level/lib/lod/level-lod-options";

describe("level LOD options", () => {
  it("draws at the game's own distance by default", () => {
    expect(toLevelRendererLod(DEFAULT_RENDERER_LOD_SETTINGS, DEFAULT_LEVEL_LOD_OPTIONS, true)).toEqual(
      DEFAULT_RENDERER_LOD_SETTINGS
    );
  });

  // A clump's screen area falls with the square of its distance, so twice as far takes four times the detail scale.
  it("scales the engine's detail by the square of the distance asked for", () => {
    expect(toLevelRendererLod(DEFAULT_RENDERER_LOD_SETTINGS, { distance: 2 }, true).geometryLod).toBeCloseTo(
      DEFAULT_RENDERER_LOD_SETTINGS.geometryLod * 4
    );
    expect(toLevelRendererLod(DEFAULT_RENDERER_LOD_SETTINGS, { distance: 0.5 }, true).geometryLod).toBeCloseTo(
      DEFAULT_RENDERER_LOD_SETTINGS.geometryLod / 4
    );
  });

  it("leaves the thresholds alone and carries the toggle", () => {
    const lod = toLevelRendererLod(DEFAULT_RENDERER_LOD_SETTINGS, { distance: 2 }, false);

    expect(lod.isImpostors).toBe(false);
    expect([lod.ssaA, lod.ssaB, lod.ssaDiscard]).toEqual([
      DEFAULT_RENDERER_LOD_SETTINGS.ssaA,
      DEFAULT_RENDERER_LOD_SETTINGS.ssaB,
      DEFAULT_RENDERER_LOD_SETTINGS.ssaDiscard,
    ]);
  });

  // The toolbar narrows what the settings set: it can turn impostors off for its view, and never on against them.
  it("turns impostors off where either the settings or the toolbar does", () => {
    const off = { ...DEFAULT_RENDERER_LOD_SETTINGS, isImpostors: false };

    expect(toLevelRendererLod(off, DEFAULT_LEVEL_LOD_OPTIONS, true).isImpostors).toBe(false);
    expect(toLevelRendererLod(DEFAULT_RENDERER_LOD_SETTINGS, DEFAULT_LEVEL_LOD_OPTIONS, false).isImpostors).toBe(false);
    expect(toLevelRendererLod({ ...off, geometryLod: 1.5 }, { distance: 2 }, true).geometryLod).toBeCloseTo(6);
  });
});
