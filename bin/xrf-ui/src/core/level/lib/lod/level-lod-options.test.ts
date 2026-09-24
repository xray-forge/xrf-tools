import { describe, expect, it } from "@jest/globals";
import { DEFAULT_RENDERER_LOD_SETTINGS } from "@xrf/renderer";

import { DEFAULT_LEVEL_LOD_OPTIONS, toLevelRendererLod } from "@/core/level/lib/lod/level-lod-options";

describe("level LOD options", () => {
  it("draws at the game's own distance by default", () => {
    expect(toLevelRendererLod(DEFAULT_LEVEL_LOD_OPTIONS, true)).toEqual(DEFAULT_RENDERER_LOD_SETTINGS);
  });

  // A clump's screen area falls with the square of its distance, so twice as far takes four times the detail scale.
  it("scales the engine's detail by the square of the distance asked for", () => {
    expect(toLevelRendererLod({ distance: 2 }, true).geometryLod).toBeCloseTo(
      DEFAULT_RENDERER_LOD_SETTINGS.geometryLod * 4
    );
    expect(toLevelRendererLod({ distance: 0.5 }, true).geometryLod).toBeCloseTo(
      DEFAULT_RENDERER_LOD_SETTINGS.geometryLod / 4
    );
  });

  it("leaves the thresholds alone and carries the toggle", () => {
    const lod = toLevelRendererLod({ distance: 2 }, false);

    expect(lod.isImpostors).toBe(false);
    expect([lod.ssaA, lod.ssaB, lod.ssaDiscard]).toEqual([
      DEFAULT_RENDERER_LOD_SETTINGS.ssaA,
      DEFAULT_RENDERER_LOD_SETTINGS.ssaB,
      DEFAULT_RENDERER_LOD_SETTINGS.ssaDiscard,
    ]);
  });
});
