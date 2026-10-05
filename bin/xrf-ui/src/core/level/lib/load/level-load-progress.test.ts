import { describe, expect, it } from "@jest/globals";

import { RenderLoadReport } from "@/core/ipc/types/xrf-renderer";
import { describeLevelLoad } from "@/core/level/lib/load/level-load-progress";

/**
 * @param load - The fields that differ from a level whose sectors and textures are all in.
 * @returns A report short of ready.
 */
function mockLoad(load: Partial<RenderLoadReport>): RenderLoadReport {
  return {
    sectors: 118,
    sectorsTotal: 118,
    bytes: 0,
    textures: 1062,
    texturesTotal: 1062,
    isReady: false,
    ...load,
  };
}

describe("describeLevelLoad", () => {
  it("counts sectors while any is still being read", () => {
    expect(describeLevelLoad(mockLoad({ sectors: 28, textures: 258, texturesTotal: 310 }))).toBe(
      "Reading sectors, 28 of 118"
    );
  });

  it("counts textures once every sector is in", () => {
    expect(describeLevelLoad(mockLoad({ textures: 599, texturesTotal: 636 }))).toBe("Uploading textures, 599 of 636");
  });

  // Grass, lights and particles are read beside the sectors, and on a debug build the grass comes last by seconds.
  it("says what is still read beside the sectors once they and their textures are all in", () => {
    expect(describeLevelLoad(mockLoad({}))).toBe("Reading spawned objects, grass, lights and particles");
  });
});
