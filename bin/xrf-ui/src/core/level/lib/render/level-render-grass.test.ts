import { describe, expect, it } from "@jest/globals";
import { ERendererDraw, IRendererGrass } from "@xrf/renderer";

import { LevelDetailsDescription } from "@/core/ipc/types/xrf-app";
import { toLevelRendererGrass } from "@/core/level/lib/render/level-render-grass";
import { ILevelGrassDelivery } from "@/core/level/lib/render/level-render-protocol";
import { mockLevelDetailsDescription } from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

function mockGrass(): ILevelGrassDelivery {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description: LevelDetailsDescription = mockLevelDetailsDescription(buffer);

  return { buffer: buffer.toArrayBuffer(), description };
}

describe("toLevelRendererGrass", () => {
  it("reads the grid, slots, bins and triangles out of the pack, and each model with its texture cut out", () => {
    const grass: IRendererGrass = toLevelRendererGrass(mockGrass());

    expect(Array.from(grass.grid)).toEqual([1, 0]);
    expect(Array.from(grass.slots)).toEqual([1, 2, 3, 4, 0, 1]);
    expect(Array.from(grass.bins)).toEqual([0]);
    expect(Array.from(grass.triangles)).toEqual([0, 1, 0, 2, 1, 0, 0, 1, 2]);
    expect([grass.sizeX, grass.sizeZ, grass.offsetX, grass.offsetZ]).toEqual([2, 1, 3, -2]);

    const [model] = grass.models;

    expect(Array.from(model.indices)).toEqual([0, 2, 1]);
    expect(Array.from(model.positions)).toEqual([0, 0, 0, 1, 0, 0, 0, 2, 0]);
    expect(Array.from(model.uvs)).toEqual([0, 1, 1, 1, 0.5, 0]);
    expect(model.surface.draw).toBe(ERendererDraw.CUT_OUT);
    expect(model.surface.textures.base).toBe("detail\\grass");
    expect([model.isWaving, model.minScale, model.maxScale, model.height, model.radius]).toEqual([
      true,
      0.5,
      1.5,
      2,
      1.2,
    ]);
  });

  // The renderer takes what it is handed, so the loader's own pack must survive for a renderer started later.
  it("reads a copy, leaving the loader's pack its own", () => {
    const delivery: ILevelGrassDelivery = mockGrass();
    const grass: IRendererGrass = toLevelRendererGrass(delivery);

    expect(grass.grid.buffer).not.toBe(delivery.buffer);
  });
});
