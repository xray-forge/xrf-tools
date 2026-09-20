import { describe, expect, it } from "@jest/globals";

import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { createSectorViews } from "@/core/level/lib/level-sector-views";
import { EMPTY_LEVEL_STATS, ILevelStats, measureLevelStats } from "@/core/level/lib/level-stats";
import { IRenderFrameCost } from "@/core/render/lib/render-viewport";
import {
  mockSectorDescription,
  mockSectorInstanceGroup,
  mockSectorSection,
  mockSectorSurface,
} from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

/** What a renderer counted for the frame just drawn. */
function frameCost(overrides: Partial<IRenderFrameCost> = {}): IRenderFrameCost {
  return { draws: 0, frameTime: 0, framesPerSecond: 0, triangles: 0, ...overrides };
}

/** One resident sector, without a renderer to upload it. */
function loadedSector(sector: number, sections: number): ILoadedSector {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description = mockSectorDescription(buffer, {
    instances: [mockSectorInstanceGroup(buffer, [0, 100])],
    sector,
    sections: Array.from({ length: sections }, (_, index) =>
      mockSectorSection({ draw: { count: 3, start: index * 3 }, surface: mockSectorSurface({ shaderId: index }) })
    ),
  });

  return {
    geometry: { dispose: () => undefined } as never,
    sector,
    views: createSectorViews({ ...description, bufferLength: buffer.byteLength }, buffer.toArrayBuffer()),
  };
}

describe("level stats", () => {
  it("measures nothing when nothing is resident and nothing has been drawn", () => {
    expect(measureLevelStats(new Map(), frameCost())).toEqual(EMPTY_LEVEL_STATS);
  });

  // What is held is the residency budget's question, and the only one the sectors themselves answer.
  it("counts the sectors held and the bytes they weigh", () => {
    const sectors: Map<number, ILoadedSector> = new Map([
      [0, loadedSector(0, 2)],
      [1, loadedSector(1, 3)],
    ]);

    const stats: ILevelStats = measureLevelStats(sectors, frameCost());

    expect(stats.sectors).toBe(2);
    expect(stats.bytes).toBeGreaterThan(0);
  });

  // What a frame cost is the renderer's question, and the two stopped agreeing the moment anything was culled:
  // counting the held draws and calling that the frame's cost reports the number culling exists to reduce.
  it("takes the draws and triangles from what the renderer counted, not from what is held", () => {
    const sectors: Map<number, ILoadedSector> = new Map([[0, loadedSector(0, 40)]]);

    const stats: ILevelStats = measureLevelStats(sectors, frameCost({ draws: 7, triangles: 120 }));

    expect(stats.draws).toBe(7);
    expect(stats.triangles).toBe(120);
  });

  it("reports the frame timing the viewport measured", () => {
    const stats: ILevelStats = measureLevelStats(new Map(), frameCost({ frameTime: 20, framesPerSecond: 50 }));

    expect(stats.frameTime).toBe(20);
    expect(stats.framesPerSecond).toBe(50);
  });
});
